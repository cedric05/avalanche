
# TODO multi
- [ ] single path or multiple path (prefer multipath)

# TODO Request
- [ ] Url query params forwarding with custom params
- [ ] Headers with mut with forwarding
- [ ] Remove hop headers
- [ ] Add logging for each request with invoked user into file (configurable)
- [ ] provide stats on which api is called and how many times (promotheues)

# TODO Request  AUTH
- [ ] Basic Auth
- [ ] Header Auth
- [ ] Aws auth
- [ ] digest auth???
- [ ] save header information safely

# TODO Response
- [ ] remove few headers from response
- [ ] transform (json to xml, xml to json and few other)

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

# TODO Docs
 

# TODO tests


refer or inspire headers to discard from https://github.com/felipenoris/hyper-reverse-proxy/blob/master/src/lib.rs