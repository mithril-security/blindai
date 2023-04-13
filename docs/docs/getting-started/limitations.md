Confidential Computing technologies have many great security features, but they also have their limitations. In this section, we’ll explore these limitations and how they apply to BlindAI API and Core.

!!! important "Audit: BlindAI Core"

A beta version of the BlindAI Core solution has been independently audited by [Quarkslab SAS](https://www.quarkslab.com/fr/accueil/). We will release their audit report in the near future.

This audit does **not cover** the **client-side SDK**, the **BlindAI API** or **Nitro enclaves**.
Some information-level security weaknesses were identified, such as insufficient error handling or the inclusion of dependencies that are no longer officially maintained. See the audit report for an exhaustive list.

### General

We must trust the official application code! We verify that the enclave is running the official BlindAI server application during attestation, but enclaves don’t audit the application code itself. This is why BlindAI is open-source, so you can audit our code yourself or refer to the previously mentioned Quarkslab audit report.
Zero-day attacks, when hackers exploit previously unknown flaws before developers have an opportunity to fix the issue, are always a risk, even with enclaves. We mitigate this risk by keeping BlindAI up-to-date with the security updates of our dependencies.

### Intel SGX specific

The verification of the enclave during the attestation process relies on the manifest.toml in the client package being authentic. If a malicious party were to succeed in tampering with this manifest.toml, this verification process could be circumvented.
Intel SGX shields the enclave from the host machine, but does not shield the host machine from the enclave. We must trust the official BlindAI enclave application code because it can interfere with the host machine.
Previous attacks on Intel SGX structures have largely consisted of side-channel attacks. Side-channels attacks look to gather information from an enclave application by measuring or exploiting indirect effects of the system or its hardware rather than targeting the program or its code directly. They are detailed in the Quarkslab’s audit report (we will update this page to include the link to that report as soon as it is released). Note that we keep up-to-date with Intel SGX security patches and no similar vulnerabilities were identified in BlindAI's audit.

!!! warning

We are currently aware of a bug that enables denial of service attacks by uploading large numbers of models to the server instance.

### Nitro Enclaves specific

We must trust AWS as the cloud provider, their hardware and the enclave’s OS. Nitro enclaves are designed to separate and isolate the host from the enclave and vice versa, and not to protect against the cloud operator (AWS) or infrastructure. (See our [Nitro guide](https://blindai.mithrilsecurity.io/en/latest/docs/concepts/SGX_vs_Nitro/#nitro-enclaves) for more information).

!!! danger "Warning"

BlindAI does not yet support attestation for Nitro Enclaves. This is only relevant to API models using Nitro Enclaves and a warning to this effect will appear when using the client API to connect to the enclave. The attestation feature will be added to the project in the near future!

## Conclusions

In this section, we have gone over the various **security limitations of BlindAI** relating to enclaves:
We must trust the application code.
The integrity of the attestation process relies on the integrity of the BlindAI client’s built-in manifest.toml file.
Nitro enclaves do not shield the enclave against the cloud operator (AWS) or infrastructure.
SGX enclaves do not shield the host machine from the enclave application.
Side-channel attacks have previously been carried out against SGX enclaves to indirectly gain information about its contents.

For more detailed information on the security limitations of Intel SGX, look out for the BlindAI Core audit report release coming soon or check out our [threat model](../security/threat_model.md).