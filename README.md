# kplc_alerts (Kenya power and lighting company)

system for extracting and notifying subscribers of kplc planned power interruptions.

## Why I built this ?

1. I wanted to challenge myself into building an entire system, frontend, backend, DB;
2. This is a service that I actually needed because KPLC does not properly target consumers this way. One has to keep checking the website / their X account if their area of work / home will be affected by planned power interruptions.

The platform is no longer hosted, but the code is now public.

## Overall structure

- frontend-workspace -> Contains Frontend react code.
- rust-workspace -> Contains Rust backend services.

## Backend structure:

I leveraged the vertical slice architecture. In a nutshell, all the code that changes together, stays together, no matter the technical boundary it operates in. Below is a list of the different modules and what they do.

### Structure

1. scheduled_interruptions -> Extracts the data from the planned power interruptions page https://kplc.co.ke/category/view/50/planned-power-interruptions, parses it and saves this data in the relevant db tables.

2. location_subscription -> Handles the location subscription & unsubscribe logic. Also contains the logic that tells us who is affected by planned outages by the areas they subscribed to.

3. location_search -> Handles location search. I used google apis as the external provider. Since google charges for every search, we use postgres to "cache" the results once fetched. Not only do we get the exact location, but we also get nearby locations.

4. shared_kernel -> Code that is re-used accross the different member crates.

5. subscribers -> Creating / getting subscriber details & also authenticating existing subscribers.

6. notifications -> Logic that sends out notifications. (Only email is enabled for now)

7. storage -> Stores the migrations and bare minimum postgres & redis / dragonfly configurations that are then imported by the other crates for storage related operations.

8. http_server -> serve http requests.

9. background_workers -> Handles queue related tasks & also provides rate limiting functionality for both the email & location search external api calls. Internally, the task functions call the public functions from `location_search` and `notifications` in order to send out the notifications. (Perhaps I should also group the tasks in the location and notification crates, but I'll have to think this through)
