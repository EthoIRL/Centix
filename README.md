# Centix [![Rust](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml/badge.svg)](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml) 
File hosting microservice(s) that include a elegant frontend built using ASP.NET 6 & a backend using Rust 1.67. Provides an easy way to share videos and images across the web (Such as streamable or youtube but self-hosted!).

## Stability
- **Do not use** the **frontend** in production (The current state of the frontend is unfinished).
- The backend is pretty much complete and stable including the database so no need to migrate sled databases.


## Frontend pictures
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/index-online.png" width="512"/>
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/index-down.png" width="512"/>
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/account.png" width="512"/>
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/post.png" width="512"/>
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/login.png" width="256"/>
<img src="https://raw.githubusercontent.com/EthoIRL/Centix/main/pictures/signup.png" width="256"/>

## General Information
* The backend & frontend are independent from each other allowing modularity between them such as a completely different frontend.

## Backend
- A single backend centix instance can handle multiple domains e.g. (exampleA.com/HilrvkpJ/ & exampleB.com/HilrvkpJ/). Both will serve the same files & accounts but can be accessed through either domain vice versa.

## Analytics (Optional / Noninvasive)
The analytics are meant to be a tool to the administrator of the instance not for me to collect data.
[Open source analytic's by Tom Draper](https://github.com/tom-draper/api-analytics).
- The backend (Rocket.rs) is using the official [library](https://crates.io/crates/rocket-analytics)
- The frontend (ASP.NET) is using a custom **unofficial** middleware to interact with the api!

## Analytics Setup
- Goto https://www.apianalytics.dev/ and make a **free** api key!
- Add the apikey to both the frontend & backend configuration files & restart!
- It should append data to the dashboard @ https://www.apianalytics.dev/api-key-here

### Rust crate dependencies
* Utoipa / Swagger (Api)
* Rocket (Web)
* Sled (Database)
* Api-Analytics (Analytics)

Buildable on Rust >=1.67.0

Running the backend
```
git clone https://github.com/EthoIRL/Centix
cd ./Centix/backend/
cargo run --release
```
Swagger interface should be accessible at ``localhost:8000/swagger/``

## Frontend setup
The frontend must point to a Centix Backend instance however it can still run without a backend just with no functionality. (In case the backend goes down abruptly)
- First make sure to run at least once to generate the configuration file!
- Find the configuration file and add the address to your backend instance (e.g. ```"BackendApiUri": "http://127.0.0.1:8000/api"```)

### Dotnet minimum supported version
* Dotnet 6 / ASP.NET 6

Running the frontend
```
git clone https://github.com/EthoIRL/Centix
cd ./Centix/frontend/
dotnet run
```

## Configuration
Since comments in json are not supported most can be found inside the program itself.
- Backend configuration [comments](https://github.com/EthoIRL/Centix/blob/main/backend/src/config.rs)
- Frontend configuration [comments](https://github.com/EthoIRL/Centix/blob/main/frontend/Config/Config.cs)

## NOTE
This is similar to my previous project (https://github.com/StrateimTech/Open-MediaServer) which is an all-in-one solution hosting both the backend and frontend in the same application. Oh and it's completely stable for production use (I use it).