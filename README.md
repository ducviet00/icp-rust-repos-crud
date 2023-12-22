# Code Repos Rust Canister

## Features
1. Add programming language info
2. Update programming language info
3. Create new repo
4. Get repo info by repo's ID
5. Get all repos info
6. Update repo's info
7. Update repo's name
8. Update repo's description
9. Delete repo by repo's ID
10. Get all programming languages info
11. Get programming language info by its ID

## Deploy Canister

```bash
dfx start --background --clean
npm run gen-deploy
```

## Commands

### Add new programming language
```bash
dfx canister call repo_manage add_language '(
  record {
    name = "Rust";
  }
)'
```

### Update programming language info
```bash
dfx canister call repo_manage update_language '(
  0,
  record {
    name = "Rust 2021";
  }
)'
```

### Get all programming languages
```bash
dfx canister call repo_manage get_all_languages
```

### Get programming language info by its ID
```bash
dfx canister call repo_manage get_language '(0)'
```

### Create new repo
```bash
dfx canister call repo_manage create_repo '(
  record {
    language_id = 0;
  	repo_name = "ICP Repo";
  	description = "first icp rust canister";
  }
)'
```

### Get all repos info
```bash
dfx canister call repo_manage get_all_repos
```

### Get repo info by its ID
```bash
dfx canister call repo_manage get_repo '(1)'
```

### Update repo info
```bash
dfx canister call repo_manage update_repo '(1, record {
    language_id = 0;
  	repo_name = "ICP repo updated";
  	description = "updated desc";
})'
```

### Update repo's name
```bash
dfx canister call repo_manage update_repo_name '(1, "Repo name updated")'
```

### Update repo's description
```bash
dfx canister call repo_manage update_repo_description '(1, "update_repo_description called")'
```

### Delete repo by repo's ID
```bash
dfx canister call repo_manage delete_repo '(1)'
```

### Stop dfx
```bash
dfx stop
```