# Articulator

A little Rust-based server for running scripts stored on the server.
Currently supports:
 * PowerShell
 * Python
 * Shell

## Building
`git clone` to a directory, and `cargo build`. The `/scripts` directory and its contents (except for .gitignore files) will be copied to the output directory when built.

## Usage
Run `articulator [hostname]` and connect to `http://hostname` in your browser. If no `hostname` is given, Articulator will bind to `localhost:3000`. Any files in the `/scripts` directory will be listed. Clicking on a link to those files will attempt to execute that script, then return its output to the caller in the HTTP response.  

## Configuring
Articulator will look in the `/scripts` folder for scripts to run. Anything in the `ret_immediately` folder will return a generic response immediately, and then kick off the requested script after a small delay. This is useful for running scripts that might prevent the server from sending a response in a timely manner (e.g. putting the server the sleep).

## Addresses
`http://hostname/` will generate and return a small HTML page which show all the scripts in the `/scripts` folder and its subdirectories.

`http://hostname/scr` will return a JSON response with all scripts listed.

`http://hostname/scr/ScriptNameHere` will attempt to execute `ScriptNameHere`. Script names are a script's file name, minus their extension.

## Security?
Nonexistent. Do not expose this to the open internet, or Bad Things will probably happen. Wrap it up in [apache](https://httpd.apache.org/) or [nginx](https://www.nginx.com/) (which actually does have a [Windows version](http://nginx.org/en/docs/windows.html)!) and make them act as reverse proxies, and do authentication and encryption.
