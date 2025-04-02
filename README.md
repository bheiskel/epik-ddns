# Dynamic DNS with Epik
Automatically updates DNS host records with new IP.

## Environment variables

| Variable | Example | Usage |
| - | - | - |
| DDNS_SIGNATURE | 0123-4567-89AB-CDEF | Epik API dynamic dns signature for managed domain|
| DDNS_DOMAIN | www.example.org | Domain to be managed |
| DDNS_HOSTNAMES | *,www | Host name records to be updated |
| DDNS_UPDATESCHEDULE | "* */15 * * * *" | Optional cron like update schedule, defaults to update every 15 minutes |
| DDNS_DRYRUN | true | Optional flag for running without updating DNS records |


