# Logos Development Environment

**The Logos project has the mission to provide the Web3 with an autonomous, community provided, blockchain secured and enterprise grade computational layer, named sub0layer.**

This repository serves as a development environment for the Logos project.

The Logos project has a lot of reaserch and development going on, so we have decided on this structure with a separate environment where only the development will take place and where no productive implementations will reside.

The approach behind this is to keep the productive implementations separate, simple, clean and allow an easier understanding of the end result.

Each of the productive implementations will be developed in this environment, tested and then rolled out in a controlled way.
This leads to a clean and secure productive environment that contains only the required implementations and thereby simplifies the complexity of the repository structure.

## Project structure
> [!NOTE]  
> Since the project is in the early stages, the structure of the project may change or may be extended.

The project structure currently consists of 3 main "segments"

- Chain (/node; /runtime; /pallet; cargo workspace):  
In this section, the blockchain (blockchain logic (runtime) and the infrastructure (client)) is configured, as the cargo workspace and the dependencies.

- Contracts (/contracts):  
As the implementation of the Logos network will mainly be carried out by smart contracts, the development in the smart contract field plays an important role for the project.

- General (.gitignore, rust-toolchain.toml, rustfmt.toml, docker, etc.):
For a clean code and project some general configurations are necessary.

## Dependency structure
The dependencies are all defined and versioned in the main cargo manifest (/Cargo.toml) and are then implemented where required.
The approach behind this is to make versioning clearer and to enable easier dependency management.

## Nightly vs Stable
In the Rust programming language, there are various "release channels" (stable, nightly, ...) that determine how often and in what form new versions and updates are made available.

The **Stable** channel is Rust's main release line. Versions in the stable channel are released every six weeks and have undergone strict quality control.

The **Nightly** channel, on the other hand, is a pre-release version of Rust which, as the name suggests, is updated daily. Nightly builds contain the very latest features and changes that are not yet available in the stable channel.

Stable is usually the best choice for production-ready applications, while Nightly is a good option for research, development of new Rust features or for projects that rely on the very latest language features.

We choose the stable channel because we do not rely on the latest Rust features, and we prefer to ensure sufficient stability before implementing updates.

## Support and Contribution
If you have any questions or would like to support or assist us in any way, you can simply contact us via the email address tech.support@logoslabs.io, open a [GitHub issue](https://github.com/logoslabstech/logos-resources/issues/new/choose), or ask anything on [Discord](https://discord.com/channels/840352211602374657/1242178591744200804).

## Preparation for Development :

[**Workstation configuration**](https://docs.logoslabs.io/development/dev-env/development-workstation)   
[**Local environment configuration**](https://docs.logoslabs.io/development/dev-env/dev-env-config)   

## Ressources
[Logos Network concept paper](https://logoslabs.io/concept/logos-network-concept-paper.pdf)   
[Logos website](https://logoslabs.io)   
[Logos documentation](https://docs.logoslabs.io)   
[Logos blog](https://blog.logoslabs.io/)   

### **We look forward to your contributions and insights as we work together to shape the future of the Web3!**