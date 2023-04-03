### Verifying code integrity checks in BlindAI

The code integrity checks are part of the remote attestation process in BlindAI. It is a key security check which checks that the BlindAI application running in our server instance has not been tampered with.

Let's go over how this check works:

1. Whenever you build BlindAI's server, a `manifest.toml` file is created which contains information relating to the configuration of the enclave and the code running inside it.

2. The client has a copy of this `manifest.toml`. BlindAI has a built-in copy of the `manifest.toml` of our latest application release code by default.

3. Whenever a user connects to a BlindAI server instance using the BlindAI, they receive a signed attestation report which contains a hash of the enclave's identity. This is verified against the client's copy of the manifest.toml. If **any** of the code inside the enclave has changed- this check will fail and the user will not be able to connect to the enclave.

### How we can verify this feature

1. First of all, you will need to be on a SGX2-ready machine for this test to work. We recommend using the Azure DCsv3 VM. For information on how to set up your Azure DCsv3 VM or how to set up BlindAI on a different SGX2-ready device, check out our [installation guide](../tutorials/core/installation.md).

2. You will need to download the `blindai` github repo and move into the root of this repo:
```bash
git clone github.com/mithril-security/blindai && cd blindai
```

3. Next, you need to modify some of the application code in any way you wish. The application source code is kept in the `.rs` files in the `src` folder. For example you can add the comment: `//testing` in any of these files.

[LAURA COMMENT: We need to wait until Andre has made this script]
4. You can build and launch your modified code. This will create a new `manifest.toml` file. 
```bash
sh build_launch_script.sh
```

5. Now you can try to connect to your server instance using the BlindAI Python library. 

Install the latest BlindAI PyPi package if you haven't already:

```
bash
pip install blindai
```

Then you can run the following code in Python:

```
python
import blindai 

blindai.core.connect(addr="localhost", hazmat_http_on_untrusted_port=True)
```

Since the BlindAI library is configured to check the enclave against the default `manifest.toml`, the client will not be able to connect to this modified server instance and will generate an error. This confirms that users **cannot** connect to a BlindAI instance that has been tampered with.