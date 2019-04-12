# Vehicle Information Service Client

This is a client implementation for the [Vehicle Information Service standard](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html).

# NOTICE

Before you use the program in productive use, please take all necessary precautions,
e.g. testing and verifying the program with regard to your specific use.
The program was tested solely for our own use cases, which might differ from yours.

# Example
The "get"-example will run a get request, asking for the value of "Private.Example.Interval".
Make sure the vehicle-communication-service example server is running and run.
```
cargo run --example get

> cargo run --example get
    Finished dev [unoptimized + debuginfo] target(s) in 0.27s
     Running `/tmp/vehicle-information-service/target/debug/examples/get`
Interval: 1
```

# Code of Conduct

Please read our [Code of Conduct](https://github.com/Daimler/daimler-foss/blob/master/CODE_OF_CONDUCT.md) as it is our base for interaction.

# Provider Information

Please visit <https://www.daimler-tss.com/en/imprint/> for information on the provider.