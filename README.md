# Centix [![Rust](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml/badge.svg)](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml) 
File hosting microservice(s) that include a elegant frontend built using ASP.NET 6 & a backend using Rust 1.67. Provides an easy way to share videos and images across the web (Such as streamable or youtube but self-hosted!).

# EARLY STAGES
* **DO NOT USE THIS IN PRODUCTION**, the backend database can change at any time!
* The backend is mostly fleshed out and function, you can access it using a the embedded swagger panel provided.
* Frontend is not complete yet!

## General Information
* The backend & frontend are independent from each other allowing modularity between them such as a completely different frontend.

## Backend
- A single backend centix instance can handle multiple domains e.g. (exampleA.com/HilrvkpJ/ & exampleB.com/HilrvkpJ/). Both will serve the same files & accounts but can be accessed through either domain vice versa.

### Rust crate dependencies
* Utoipa / Swagger (Api)
* Rocket (Web)
* Sled (Database)

Buildable on Rust >=1.67.0

Running the backend
```
git clone https://github.com/EthoIRL/Centix
cd ./Centix/backend/
cargo run --release
```
Swagger interface should be accessible at ``localhost:8000/swagger/#/``

## Frontend
- Must point to a singular centix backend instance.

### Dotnet dependencies
* Dotnet 6 / ASP.NET 6

Running the frontend
```
git clone https://github.com/EthoIRL/Centix
cd ./Centix/frontend/
dotnet run
```

## NOTE
This is similar to my previous project (https://github.com/StrateimTech/Open-MediaServer) which is an all-in-one solution hosting both the backend and frontend in the same application.

This is also learning project for me with rust, any help is appreciated.