# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

# load environment variables from .env file
set dotenv-load

# The user password to use to login to the pi
pi_pw := env_var("PI_PASSWORD")

# run the server locally
run-server:
    cargo run

# Cross compile to raspberry pi 4
build-pi:
    cross build --release --target armv7-unknown-linux-gnueabihf

# Copy the built artifact to the pi over ssh
distribute-pi:
    pscp -pw {{pi_pw}} "C:/Git/piapps/target/armv7-unknown-linux-gnueabihf/release/appserver" lando@raspberrypi:Server
    pscp -r -pw {{pi_pw}} "C:/Git/piapps/appserver/assets" lando@raspberrypi:Server
