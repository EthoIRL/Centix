# Centix [![Rust](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml/badge.svg)](https://github.com/EthoIRL/Centix/actions/workflows/rust.yml)
Image/video hosting backend server application aiming for simple but rich feature support for both picture & video distribution across the web.

## NOTE
This is similar to my previous project (https://github.com/StrateimTech/Open-MediaServer) which is an all-in-one solution hosting both the backend and frontend in the same application, on the other hand this project is focused on a more modular approach to the issue raised by separating both the frontend and backend allowing for a greater emphasis on multi client application support (web, windows, android, etc.).

### Further up
A little more information on this project
* The backend server is its own independent program. To fully utilize it you need a frontend client to access the APIs, although a Swagger web interface is provided for ease of development.
* At the current state, this project is semi-functional, but shouldn't be used for production as the APIs are constantly changing. 
* This project has similar goals to the production product of Open-MediaServer although with a client-server approach.
* For the future, maybe make a frontend such as a desktop application or web server frontend?

## Main dependencies
* Utoipa / Swagger
* Rocket
* Sled

## Other
This is a learning project for me with rust, any help is appreciated.