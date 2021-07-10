# curite
nickserv account verification URL bot

## how does this work
this will create a connection to your IRC network and spin up a listening
socket, over which it will accept HTTP POSTs including an account name and an
email verification token in the request path, which it will then give to
NickServ and try to read verification results back, returning the user to
either a static success page or a static failure page.
