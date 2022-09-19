# Ensuring privacy with BlindAI

In this section we will explained how BlindAI secure its users' data and what you should do to secure your models and your data using BlindAI.

## What is confidential computing ?

Confidential computing refers to the technology that can isolate a process within a protected CPU. During its executing the program runs into a **TEE** (Trusted Execution Environment). This is because nobody, not even the machine owner, can access in any way this environment, meaning that any sensible data, the source code and the program computations are isolated.

To achieve this we are relying on Intel SGX enabled CPUs. These CPUs have the ability to start a **Secure Enclave**, which is another way to say that it can execute code inside a TEE.

## Trusting BlindAI

As a user wanting privacy guarantees, here is a step-by-step list of what you should do to securely deploy or connect to BlindAI:

- Inspect the commit of the BlindAI instance, and make sure that data is not exposed. If you don’t want to, it's ok, we will have external independent auditors do it for you.
- Build the commit's enclave, and generate its policy.toml, then pass it to the client.
- If you're deploying your own BlindAI instance you should also generate new TLS certificates.

The next sections explain how to achieve the last two steps.
