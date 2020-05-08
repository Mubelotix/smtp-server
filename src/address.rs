use string_tools::{get_all_before, get_all_after};

#[derive(Debug, PartialEq)]
pub struct EmailAdress {
    pub username: String,
    pub domain: String
}

impl std::fmt::Display for EmailAdress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO quoted strings
        write!(f, "{}@{}", self.username, self.domain)
    }
}

impl std::str::FromStr for EmailAdress {
    type Err = &'static str;

    fn from_str(mut full_address: &str) -> Result<EmailAdress, Self::Err> {
        full_address = full_address.trim();
        let mut username = get_all_before(full_address, "@").to_string();
        let domain = get_all_after(full_address, "@").to_string();

        if username.starts_with('"') && username.ends_with('"') && username.len() >= 2 {
            username.remove(0);
            username.remove(username.len() - 1);

            let mut invalid = false;
            username.chars().for_each(|c| if c == '\\' && c == '"' {invalid = true} );

            if invalid {
                return Err("Quoted username is not valid");
            }
        } else {
            let mut last_was_point = false;
            let mut invalid = false;
            
            username.chars().for_each(|c| {
                if c == '.' && !last_was_point {
                    last_was_point = true;
                } else if c.is_ascii_alphanumeric() || "!#$%&'*+-/=?^_`{|}~".contains(c) {
                    last_was_point = false;
                } else {
                    invalid = true;
                }
            });

            if invalid {
                return Err("Username is not valid");
            }
        };

        // TODO support adress litteral
        let mut last_was_point = false;
        let mut invalid = false;
        domain.chars().for_each(|c| {
            if c == '.' && !last_was_point {
                last_was_point = true;
            } else if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                last_was_point = false;
            } else {
                invalid = true;
            }
        });

        if invalid {
            return Err("Domain name is not valid");
        }

        Ok(EmailAdress {
            username,
            domain
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn adress_parsing() {
        assert_eq!(Ok(EmailAdress {
            username: String::from("mubelotix"),
            domain: String::from("mubelotix.dev"),
        }), "mubelotix@mubelotix.dev".parse());

        assert_eq!(Ok(EmailAdress {
            username: String::from("mubelotix"),
            domain: String::from("mubelotix.dev"),
        }), "  mubelotix@mubelotix.dev ".parse());

        assert_eq!(Ok(EmailAdress {
            username: String::from("mubelotix.test"),
            domain: String::from("mubelotix.dev"),
        }), "mubelotix.test@mubelotix.dev".parse());

        assert_eq!(Err("Username is not valid"), "mubelotix..test@mubelotix.dev".parse::<EmailAdress>());
        assert_eq!(Ok(EmailAdress {
            username: String::from("mubelotix..test"),
            domain: String::from("mubelotix.dev"),
        }), " \"mubelotix..test\"@mubelotix.dev".parse());

        assert_eq!(Err("Domain name is not valid"), "mubelotix@mubelotix..dev".parse::<EmailAdress>());
        assert_eq!(Err("Domain name is not valid"), "mubelotix@mubeLotix.dev".parse::<EmailAdress>());
    }
}