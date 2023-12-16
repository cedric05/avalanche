# Avalanche

This project is a Rust application that provides functionality for managing projects and handling project requests. It includes features for reading project configurations from a file, configuring services, and managing users and authentication tokens.

## Features

- **File-Based Project Configuration**: Projects are configured based on a JSON file. This makes it easy to set up new projects and modify existing ones.
- **Project Management**: The application includes a `FileProjectManager` that is responsible for managing projects.
- **User Management**: The application includes a `UserStore` for storing user information and a `SimpleUserTokenStore` for storing user tokens.
- **Authentication**: The application includes an `AuthTokenStore` for storing authentication tokens and a `MarsAuth` struct for handling Mars authentication.

## Getting Started

To get started with this project, you will need to have Rust installed on your machine. You can then clone the repository and run the application with the following commands:

```bash
git clone https://github.com/username/projectname.git
cd projectname
cargo run




# Contributing
Contributions are welcome! Please feel free to submit a pull request.

# License
This project is licensed under the MIT License - see the LICENSE file for details.
