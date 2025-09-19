# iRPC
Robotic node interaction protocol

## Features

- **`arm_api`**: Enables features for std host environments with async runtime and logging support
- **`joint_api`**: Enables features for no_std embedded environments  

## Usage

### Host Environment (with `arm_api` feature)
```toml
[dependencies]
irpc = { version = "0.1", features = ["arm_api"] }
```

### Embedded Environment (with `joint_api` feature)  
```toml
[dependencies]
irpc = { version = "0.1", features = ["joint_api"] }
```

### Both Environments
```toml
[dependencies]
irpc = { version = "0.1", features = ["arm_api", "joint_api"] }
```
