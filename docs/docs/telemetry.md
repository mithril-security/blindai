# Telemetry

### Why are we collecting telemetry data? <a href="#what-kind-of-data-do-we-collect" id="what-kind-of-data-do-we-collect"></a>

We are collecting those data solely to improve our project and see how BlindAI is used, to improve it and keep working on it full-time.

In order to do all of this, we need some data to see which features are the most used and if BlindAI is used in hardware or software mode. Having those data shows us on what we should focus to give the best experience possible to our users. In addition, it proves to our investors that our project is being used, allowing us to keep the project alive.

Telemetry also helps us with performance issues on the wild.

### The data we are collecting <a href="#what-kind-of-data-do-we-collect" id="what-kind-of-data-do-we-collect"></a>

BlindAI collects anonymous data regarding general usage, this allows us to understand how you are using the project and how we can improve it.

This feature can be disabled at any time and any collected data can be deleted on request.

We are currently collecting or may collect in the future:

* the platform the server or client is running on (OS type, version and release, architecture)
* version of the server and clients, and client user agents
* running mode (hardware, simulation, dcsv3)
* user identifier generated at client/server startup
* how long did a request take and other performance information
* model sizes and names
* uptime of the server and time of the event
* information about the hardware, like number of cores available, memory, ...

We will never collect these kind of data:

* User identity
* Collect personal information such as IP addresses, email addresses, website URLs
* Model or data uploaded to BlindAI

### Disable Telemetry <a href="#what-kind-of-data-do-we-collect" id="what-kind-of-data-do-we-collect"></a>

Telemetry can be disabled at any time very easily:

#### Setting up the appropriate variable environment (if you are using Docker 🐳)

```
-e BLINDAI_DISABLE_TELEMETRY=1
```

#### Setting up the appropriate variable environment (if you are building the project from source)

```bash
export BLINDAI_DISABLE_TELEMETRY=1
```

### How does the Telemetry work? <a href="#exhaustive-list-of-all-collected-data" id="exhaustive-list-of-all-collected-data"></a>

We are using [Amplitude](https://amplitude.com) to collect and see the Telemetry. It is a very powerful tool to highlight data and present them in the way we want.

The server checks every 2 seconds if there was any new event (if the server started, if a model was uploaded, if the model was run), and if it is the case, the data is sent to Amplitude.

