# curite

__Email verification web server for Atheme IRC services__

`curite` allows users to verify their email addresses by clicking a link in the verification email.
It accepts HTTP requests over a Unix socket (which requires a reverse proxy to sit between it and the internet)
and processes them using Atheme's XMLRPC interface.

## Configuration

The included configuration and [Tera](https://keats.github.io/tera/docs/) templates are used for Libera.Chat,
but they can be adapted for any IRC network using Atheme services.
The configuration can be reloaded by sending SIGHUP to a running instance of `curite`.

* `listen`: The path to the Unix socket that `curite` will listen on. `curite` expects this socket to already exist.
* `xmlrpc`: The URL of Atheme's XMLRPC endpoint.
* `templates`: A glob indicating where to find templates for the verification page.
* `verify`: URLs related to verification. Requires the following values:
  * `success`: The URL to redirect to upon successful verification.
  * `failure`: The URL to redirect to upon failed verification.
  * `target`: The URL to which a user should `POST` to verify their account. The following substrings will be replaced:
      * `{{account}}`: The account name to verify.
      * `{{token}}`: The verification token.
* `validation`: Regexes to validate fragments of the validation URL. Requires the following values:
  * `account`: A regex to validate account names.
  * `token`: A regex to validate the verification token.

## Security

`curite` does not currently support any form of human verification.
As a result, links to its pages should not be provided directly to the user over IRC.

It should be noted that Atheme's XMLRPC interface is extremely powerful and
**must not be directly exposed to the internet**.
