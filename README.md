# Lead generation management

Send emails to potential leads.


## Features

If a lead does not respond within one day, sends a follow up email.

When the lead replies, generate an automated AI response.


## App Highlights

- `sqlx` integration for easy setup and migrations.
- SQLite for easy local and cloud deploys.

## Install

### Requirements

Requires **SQLite**. On macOs / Linux SQLite is available using [HomeBrew](https://formulae.brew.sh/formula/sqlite).


### Using releases

Go to the release pages and download the latest version for your operating system.

Extract the file and run:


### Local builds

Clone the repository and run

```
[RUST_LOG=debug] cargo run
```


## Usage example

Start by generating a lead with

```
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"name":"John Doe","email":"john.doe@example.com"}' \
    http://localhost:3010/lead

# {"id":1,"name":"John Doe","email":"john.doe@example.com","phone":null}%
```

Send a message to the lead
```
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"lead_id":1, "message": "Hi John! Open to quick chat to discuss an amazing business opportunity?"}' \
    http://localhost:3010/send 

# {"id":1,"leads_id":1,"message_sent":"Hi John! Open to quick chat to discuss an amazing business opportunity?","sent_at":null,"reply_received":null,"reply_received_at":null,"ai_reply":null,"ai_reply_sent":null,"created_at":"2026-01-16T20:20:00.381430+00:00","status":"enqueued","follow_up_at":null,"closed_at":null}
```

Mock the users reply:

```
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"message_id":1,"reply":"Interested!"}' \
    http://localhost:3010/reply 

# {"id":1,"leads_id":1,"message_sent":"Hi John! Open to quick chat to discuss an amazing business opportunity?","sent_at":"2026-01-16T20:21:00.092458+00:00","reply_received":"Interested!","reply_received_at":"2026-01-16T20:21:26.330781+00:00","ai_reply":null,"ai_reply_sent":null,"created_at":"2026-01-16T20:20:00.381430+00:00","status":"replied","follow_up_at":null,"closed_at":null}
```

Generate the AI reply:

```
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"message_id":1}' \
    http://localhost:3010/ai/reply 

# {"id":1,"leads_id":1,"message_sent":"Hi John! Open to quick chat to discuss an amazing business opportunity?","sent_at":"2026-01-16T20:21:00.092458+00:00","reply_received":"Interested!","reply_received_at":"2026-01-16T20:21:26.330781+00:00","ai_reply":"Thank you for your interest! Our team will follow up shortly.","ai_reply_sent":null,"created_at":"2026-01-16T20:20:00.381430+00:00","status":"ai_enqueued","follow_up_at":null,"closed_at":null}
```

Get the history for the lead:

```
curl \
    http://localhost:3010/lead/1
```


Result will be:

```json
  "lead": {
    "id": 1,
    "name": "John Doe",
    "email": "john.doe@example.com",
    "phone": null
  },
  "messages": [
    {
      "id": 1,
      "leads_id": 1,
      "message_sent": "Hi John! Open to quick chat to discuss an amazing business opportunity?",
      "sent_at": "2026-01-16T20:21:00.092458+00:00",
      "reply_received": "Interested!",
      "reply_received_at": "2026-01-16T20:21:26.330781+00:00",
      "ai_reply": "Thank you for your interest! Our team will follow up shortly.",
      "ai_reply_sent": "2026-01-16T20:23:00.172992+00:00",
      "created_at": "2026-01-16T20:20:00.381430+00:00",
      "status": "ai_replied",
      "follow_up_at": null,
      "closed_at": null
    }
  ],
  "outreach_logs": [
    {
      "id": 5,
      "message_id": 1,
      "log_at": "2026-01-16T20:23:00.174379+00:00",
      "step": "ai_replied"
    },
    {
      "id": 4,
      "message_id": 1,
      "log_at": "2026-01-16T20:22:34.405840+00:00",
      "step": "ai_enqueued"
    },
    {
      "id": 3,
      "message_id": 1,
      "log_at": "2026-01-16T20:21:26.331588+00:00",
      "step": "replied"
    },
    {
      "id": 2,
      "message_id": 1,
      "log_at": "2026-01-16T20:21:00.095722+00:00",
      "step": "sent"
    },
    {
      "id": 1,
      "message_id": 1,
      "log_at": "2026-01-16T20:20:00.381867+00:00",
      "step": "enqueued"
    }
  ]
}
```

## Development


### Build

```sh 
/usr/bin/env bash ./cross_build.sh
```

## TODO


### Business logic
- [ ] add multiple replies and ai replies per lead. Currently only one flow is available: send email -> lead reply -> ai reply.
- [ ] when receiving a lead reply, auto-generate the ai enqueued reply.
- [ ] logic review: due to time constraints actual follow up and lead closing was not tested.

### Features
- [ ] add multiple emails between user and / AI.
- [ ] add initial email generation with AI.
- [ ] add actual email service.
- [ ] add actual AI endpoint.
- [ ] support for rich text messages in the payloads.
- [ ] add deploy mechanism (taking).

### Code improvements
- [ ] add tests.
- [ ] add postgres support.
- [ ] add external logging system (sentry, New Relic, etc).


