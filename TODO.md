
# TODO multi
- [x] single path or multiple path (prefer multipath)

# TODO Request
- [x] Url query params forwarding with custom params
- [x] Headers with mut with forwarding
- [x] Remove hop headers
- [ ] Add logging for each request with invoked user into file (configurable)
- [ ] provide stats on which api is called and how many times (promotheues)

# TODO Request  AUTH
- [x] dynamic load services for clients on service_config
- [x] Basic Auth
- [x] Header Auth
- [x] Aws auth (https://github.com/awslabs/aws-sdk-rust/blob/main/sdk/aws-sig-auth/src/lib.rs)
- [x] digest auth???
- [x] save header information safely
- [x] x503 authentication
- [ ] soap basic authentication
- [ ] soap x503 authentication
- [x] hawk auth
- [ ] oauth
- [ ] jwt auth
- [ ] NTLM
- [x] No auth

# TODO Client settings
- [ ] Set limits on client
  - [x] Timeout
  - [ ] Rate limits
  - [x] Concurrency limit per service
- [ ] Discard client if not used for last couple of hours (lower memory)
- [ ] Use database instead of settings

# TODO Response
- [x] remove few headers from response
- [x] transform (json to xml, xml to json and few other)

# TODO ADMIN
- [ ] Admin user
- [ ] Create Admin user, Create Api User
- [ ] Give access to  specific api group for a specific user
- [ ] update user access to group, delete user for a group
- [ ] Using Admin user, normal apis should not work (to safe gaurd)
- [ ] Authentication of all apis (either admin and api)
- [ ] Admin can either be launched in same or other server
- [ ] add config for a specific addresss (priviliged with write access)
- [ ] develop a layer to save config to database, in memory, from file (configuration), redis
- [ ] integration with hashicorp vault (to save secure data)

# TODO configure
- [ ] configure base path
- [ ] support for health check and stats

# TODO Docs


# TODO tests


refer or inspire headers to discard from https://github.com/felipenoris/hyper-reverse-proxy/blob/master/src/lib.rs

