# Pythia/CACTI Tracepoint Enabling/Disabling Mechanism

## Requirements

- Pass tracepoints in a unified format to a receiver
- Receiver "lives" on an application node, presumably together with the Pythia Agent
- Pass newly enabled and disabled tracepoints/diffs between prev and current iterations

## Architecture/Structure

- Open port on receivers to receive signals from controller
- Configure controller to establish connections to receivers on startup
- Send enable/disable updates as structured/semi-structured data
  - JSON format: Requires explicit delineation between enable and disable; can be achieved with few characters however

## API Endpoints

- `enable`: Used to only enable a set of tracepoints
- `disable`: Used to only disable a set of tracepoints
- `modify`: Used to both enable and disable tracepoints based on specified data