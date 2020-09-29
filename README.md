<!-- SPDX-License-Identifier: MIT -->

# Vehicle Information Service

[![CI](https://github.com/Daimler/vehicle-information-service/workflows/CI/badge.svg)](https://github.com/Daimler/vehicle-information-service)

This is an implementation of the [Vehicle Information Service standard](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html).

# NOTICE

Before you use the program in productive use, please take all necessary precautions,
e.g. testing and verifying the program with regard to your specific use.
The program was tested solely for our own use cases, which might differ from yours.

# Code of Conduct

Please read our [Code of Conduct](https://github.com/Daimler/daimler-foss/blob/master/CODE_OF_CONDUCT.md) as it is our base for interaction.

# Provider Information

Please visit <https://www.daimler-tss.com/en/imprint/> for information on the provider.

# License Checks

This project uses [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for checking the licenses of dependencies. To run the check locally run the following:

```
cargo install cargo-deny
cargo deny check
```