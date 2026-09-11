# Security boundary

The visualizer is a read-only consumer of bounded, validated map bundles. It never obtains game mutation credentials, reads game memory, loads game assemblies, connects to the game mod, or dispatches gameplay actions. Route selection is local annotation.

Untrusted bundle input is bounded before parsing, duplicate keys and unsupported shapes are rejected, and content/identity relationships are verified before display. Input SVG/HTML is never executed. Deterministic SVG is generated from typed presentation data with escaped labels; PNG rasterization does not load system fonts or external resources.

The optional HTTP listener binds only to loopback. It uses fixed GET/HEAD routes, exact Host/origin checks, a restrictive content security policy, finite header/body/connection limits and deadlines. Artifact reads use directory capabilities to prevent path and symlink escapes, including path replacement races. No arbitrary URL proxy, upload endpoint, credential forwarding or public bind exists.

Report suspected security defects through the organization private vulnerability reporting process. Do not publish credentials, personal paths, private traces or live exploit details in an ordinary issue. Dependency and application checks are component evidence, not proof that any deployed service is safe or current.
