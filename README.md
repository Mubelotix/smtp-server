# Smtp-server

This binary crate is intended to run a full SMTP server on your computer. It is composed of an MTA (to send emails) and of an MDA (to receive emails). It fully supports TLS (you just need a pfx certificate that can be easily generated).  
  
This crate could be divided between a library crate and a binary crate in the future and crates supporting POP and IMAP should be created.  

# Goal

The goal of this crate is to provide an easy to deploy, secure, powerful, and fast mail server.  
To achieve that goal, we first need to implement:  
  
- [x] TLS
- [ ] Authentification
- [ ] DKIM
- [ ] Error handling (avoid panics)
- [ ] Multithreading (and then async)

# How to run

That crate is using clap. You can run `cargo run -- --help` or directly `./compiled_program --help` to see the full list of arguments and subcommands. Logging can be enabled by setting the `RUST_LOG` environment variable to `fatal,smtp-sever=debug`.