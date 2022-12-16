# Centix [![Rust](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml/badge.svg)](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml)
Image & Video hosting applications (Backend server infastructure, and cross platform application).

Provides an easy way to share videos and images across the web (also supports generic files).

## General Information
* The backend server is its own independent program. To fully utilize it you need a frontend client to access the APIs, although a Swagger web interface is provided for ease of development.
* At the current state, this project is semi-functional, but shouldn't be used for production as the APIs are constantly changing. 
* This project has similar goals to the production product of [Open-MediaServer](https://github.com/StrateimTech/Open-MediaServer) although with a client-server approach.

## Backend
### Info
A single backend centix instance can handle multiple domains e.g. (a.com/HilrvkpJ/ & b.com/HilrvkpJ/. both serve the same file & accounts can be acessed through either.)

### Rust crate dependencies
* Utoipa / Swagger (Api)
* Rocket (Web)
* Sled (Database)

Buildable on Rust >=1.64.0

Running the backend
```
git clone https://github.com/EthoIRL/Centix
cd ./Centix/backend/
cargo run --release
```
Swagger interface should be acessiable at ``localhost:8000/swagger/#/``

## NOTE
This is similar to my previous project (https://github.com/StrateimTech/Open-MediaServer) which is an all-in-one solution hosting both the backend and frontend in the same application.

This is also learning project for me with rust, any help is appreciated.