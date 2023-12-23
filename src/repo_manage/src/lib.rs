#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Repo {
    id: u64,
    language_id: u64,
    repo_name: String,
    description: String,
    updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Repo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Repo {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ProgrammingLanguage {
    id: u64,
    name: String,
    updated_at: Option<u64>,
}

impl Storable for ProgrammingLanguage {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for ProgrammingLanguage {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static REPO_STORAGE: RefCell<StableBTreeMap<u64, Repo, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static LANGUAGE_STORAGE: RefCell<StableBTreeMap<u64, ProgrammingLanguage, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct RepoPayload {
    language_id: u64,
    repo_name: String,
    description: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ProgrammingLanguagePayload {
    name: String,
}

#[ic_cdk::query]
fn get_repo(id: u64) -> Result<Repo, Error> {
    match _get_repo(&id) {
        Some(repo) => Ok(repo),
        None => Err(Error::NotFound {
            entity: "Repo".to_string(),
            id,
        }),
    }
}

#[ic_cdk::query]
fn get_all_repos() -> Result<Vec<Repo>, Error> {
    let repos_map: Vec<(u64, Repo)> =
        REPO_STORAGE.with(|service| service.borrow().iter().collect());
    let repos: Vec<Repo> = repos_map.into_iter().map(|(_, repo)| repo).collect();

    if !repos.is_empty() {
        Ok(repos)
    } else {
        Err(Error::NotFound {
            entity: "Repo".to_string(),
            id: 0,
        })
    }
}

#[ic_cdk::query]
fn get_language(id: u64) -> Result<ProgrammingLanguage, Error> {
    match _get_language(&id) {
        Some(language) => Ok(language),
        None => Err(Error::NotFound {
            entity: "ProgrammingLanguage".to_string(),
            id,
        }),
    }
}

#[ic_cdk::query]
fn get_all_languages() -> Result<Vec<ProgrammingLanguage>, Error> {
    let langs_map: Vec<(u64, ProgrammingLanguage)> =
        LANGUAGE_STORAGE.with(|service| service.borrow().iter().collect());
    let langs: Vec<ProgrammingLanguage> = langs_map.into_iter().map(|(_, lang)| lang).collect();

    if !langs.is_empty() {
        Ok(langs)
    } else {
        Err(Error::NotFound {
            entity: "ProgrammingLanguage".to_string(),
            id: 0,
        })
    }
}

#[ic_cdk::update]
fn create_repo(payload: RepoPayload) -> Result<Repo, Error> {
    if payload.repo_name.is_empty() {
        return Err(Error::CreateFail {
            msg: String::from("Invalid repo name"),
        });
    };
    if payload.description.is_empty() {
        return Err(Error::CreateFail {
            msg: String::from("Invalid description"),
        });
    };

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let repo = Repo {
        id,
        language_id: payload.language_id,
        repo_name: payload.repo_name,
        description: payload.description,
        updated_at: Some(time()),
    };
    do_insert_repo(&repo);
    Ok(repo)
}

#[ic_cdk::update]
fn update_repo(id: u64, payload: RepoPayload) -> Result<Repo, Error> {
    if payload.repo_name.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid repo name"),
        });
    };
    if payload.description.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid description"),
        });
    };

    match REPO_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut repo) => {
            repo.language_id = payload.language_id;
            repo.repo_name = payload.repo_name;
            repo.description = payload.description;
            repo.updated_at = Some(time());
            do_insert_repo(&repo);
            Ok(repo)
        }
        None => Err(Error::NotFound {
            entity: "Repo".to_string(),
            id,
        }),
    }
}

#[ic_cdk::update]
fn update_repo_name(id: u64, repo_name: String) -> Result<Repo, Error> {
    if repo_name.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid repo name"),
        });
    };
    match REPO_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut repo) => {
            repo.repo_name = repo_name;
            repo.updated_at = Some(time());
            do_insert_repo(&repo);
            Ok(repo)
        }
        None => Err(Error::NotFound {
            entity: "Repo".to_string(),
            id,
        }),
    }
}

#[ic_cdk::update]
fn update_repo_description(id: u64, description: String) -> Result<Repo, Error> {
    if description.is_empty() {
        return Err(Error::UpdateFail {
            msg: String::from("Invalid description"),
        });
    };
    match REPO_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut repo) => {
            repo.description = description;
            repo.updated_at = Some(time());
            do_insert_repo(&repo);
            Ok(repo)
        }
        None => Err(Error::NotFound {
            entity: "Repo".to_string(),
            id,
        }),
    }
}

// helper method to perform insert.
fn do_insert_repo(repo: &Repo) {
    REPO_STORAGE.with(|service| service.borrow_mut().insert(repo.id, repo.clone()));
}

fn do_insert_language(lang: &ProgrammingLanguage) {
    LANGUAGE_STORAGE.with(|service| service.borrow_mut().insert(lang.id, lang.clone()));
}

#[ic_cdk::update]
fn delete_repo(id: u64) -> Result<Repo, Error> {
    match REPO_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(repo) => Ok(repo),
        None => Err(Error::NotFound {
            entity: "Repo".to_string(),
            id,
        }),
    }
}

// a helper method to get a repo by id
fn _get_repo(id: &u64) -> Option<Repo> {
    REPO_STORAGE.with(|service| service.borrow().get(id))
}

// a helper method to get a programming language by id
fn _get_language(id: &u64) -> Option<ProgrammingLanguage> {
    LANGUAGE_STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn add_language(payload: ProgrammingLanguagePayload) -> Result<ProgrammingLanguage, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let language = ProgrammingLanguage {
        id,
        name: payload.name,
        updated_at: Some(time()),
    };
    do_insert_language(&language);
    Ok(language)
}

#[ic_cdk::update]
fn update_language(
    id: u64,
    payload: ProgrammingLanguagePayload,
) -> Result<ProgrammingLanguage, Error> {
    let language_option: Option<ProgrammingLanguage> =
        LANGUAGE_STORAGE.with(|service| service.borrow().get(&id));

    match language_option {
        Some(mut lang) => {
            lang.name = payload.name;
            lang.updated_at = Some(time());
            do_insert_language(&lang);
            Ok(lang)
        }
        None => Err(Error::NotFound {
            entity: "ProgrammingLanguage".to_string(),
            id,
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { entity: String, id: u64 },
    CreateFail { msg: String },
    UpdateFail { msg: String },
}

// Candid Interface
ic_cdk::export_candid!();
