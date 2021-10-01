# some-bdd
Some BDD examples using Rust Cucumber

## Cargo test

Execute the tests by running cargo test and provide the parameters shown underneath. This will run the default cucumber event handler.

```
# cargo test -- [API-HOST] [API-KEY] [SECRET-KEY] [OTP]
```

An additional parameter can be used as the output file, then the app will be able to capture cucumber events during the execution and create an output file as JSON.

```
# cargo test -- [API-HOST] [API-KEY] [SECRET-KEY] [OTP] [FILENAME]
```

## Docker build

Build the image by providing the environment parameters and avoid to provide them during the container execution:

```
# docker build -t some-bdd --build-arg api_host=[API-HOST] --build-arg api_key=[API-KEY] --build-arg secret_key=[SECRET-KEY] .
```

## Docker run

Run the container by providing the 2FA OTP as environment parameter:

```
# docker run --env OTP=[OTP] some-bdd
```

If you want to export the results, use the environment parameter OUTPUT as the output json file that will be generated at the ./out directory. Run the docker with a volume to mount file systems and get the file:

```
# docker run --env OTP=[OTP] --env OUTPUT=[FILENAME] --volume [YOUR-LOCAL-PATH]:/usr/src/somebdd/out some-bdd
```