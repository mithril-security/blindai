# Threat model
__________________________________________

This document provides the threat model for **BlindAI using the Intel SGX platform and fortanix EDP**. 

## Target Evaluation 
__________________________________________

In this threat model, the target of evaluation is the BlindAI code (using the fortanix EDP), server side **(TC1)** and client side **(TC2)**, the fortanix EDP platform **(TC3)**, the Intel SGX platform **(TC4)**, the SDK and software related to SGX DCAP (which includes AESM, the PCCS service and the intel-sgx SDK) **(TC5)**. 
*TC for target component notation*.

BlindAI can originally be configured in a simulated or hardware enabled mode. 
We consider the latter in this assesment as it is the mode that will be used in production. 
To achieve that configuration, we make the following assumptions :

- The images must be ran on an compatible SGX2 platform supporting ECDSA-attestation
- Focus on the run-time of the life-cycle (which does not including sealing, firmware issues...)
- Invasive attacks are not included as such as decapsulating (as they are not relevant in a real world environment in our case).
- Non-Invasive attacks, disregarding the possibility of having direct access to the physical machine, are included for example, side-channel attacks are highly watched. 
- The secure Boot is enabled and the intel and constructor images are up to date, and the SDK used is non-modified intel sgx. 
- The OS and the related drivers and applications are not trusted inside the SGX Enclave (Iago Attacks are possible). 
- We consider the data and model sent by the client to the server to be trusted, as he is the one that owns it. 
- The issues concerning the guarantee of the availability of the SGX platform and hence the BlindAi App are not taken into consideration and are out of scope. 

## Data Flow Diagram
__________________________________________

The figure below shows a high-level data flow diagram for the BlindAI app. The diagram shows a model of the different components that interacts to achieve the remote attestation process and running the models as an inference engine. 

- the red lines present the insecure connections. 
- the green lines show the secure connections. 
- the black ones are delivered in the dependencies used in BlindAI (Fortanix with aesm and Intel DCAP). 


![Data flow diagram](../../../assets/data_flow_diagram.png)

| Diagram Element | Description   |
| ---       | --------------|
| **DF1**   | Initiate a unsecure HTTP connection between the client API and the server. This connection must be linked to a reverse proxy that encrypts the traffic sent |
| **DF2**   | The enclave and the runner initiate a simple HTTP communication. |
| **DF3**   | Initialization of the runner context by calling to the quoting enclave via the AESM service and computing the targetInfo |
| **DF4**   | providing the targetinfo to the enclave, and returning the signed report | 
| **DF5**   | Contacting the PCCS service via the Intel DCAP API to get the collateral information |
| **DF6**   | Communicating the quote and collateral to the enclave | 
| **DF7**   | Sending the the quote and collateral through the unsecure TLS connection to be verified by the client |
| **DF8**   | Create a new TLS connection, this one using the information exchanged |  
| **DF9**   | Begin exchanging the data through the secure TLS connection to be processed by the enclave |

## Threat Analysis 
__________________________________________

In this section we provide an assessment of the potential threats to the BlindAI app while indentifying the different dependencies used and possible vectors of foothold. 

For each threat, we identify the *asset* that is under threat, the *threat agent*, and the *threat type*. 

Each threat is also given a *risk rating* that represent the impact and likelihood of that threat, and potential mitigations are also provided accordingly. 


### Assets 

| Asset | Description |
| ----- | ----------- |
| ***Sensitive Data*** | These include the sensitive data that an attacker must be able to tamper with *(e.g. Root of trust Public Key, certificates)*, see *(e.g. degugging information)* or extract *(e.g. private keys)*. |
| ***User Data*** |   These include the user data that are sent from the client to the server to be processed by a verified SGX enclave. |
| ***Model*** | It represents the model uploaded by the user in the server to be used |
| ***Code Execution*** | This represents the requirement that the platform should run only BlindAI code inside the enclave. |
| ***Availability*** | This represents the availability of the BlindAI app through it's use. |


### Threat Agents 

The threat agents represents the possible entry points that may be used by potential attackers. 


| Threat Agent | Description |
| ----- | ----------- |
|  NSCode | Represent the malicious or faulty code running on the unsecure world (it not only includes blindAI server part running outside the enclave but also the intel API, services and the dependencies that are used). |
| SecCode | Represent the malicious or faulty code running on the secure world, i.e. the SGX enclave. It includes the blindAI code running inside the enclave and the related dependencies (Fortanix API, and code dependencies). |


### Threat Types

In this Threat Model we categorize the threats using the ***STRIDE Threat analysis technique***. Thus, a threat is categorized as one or more of these types: 
`spoofing`, `Tampering`, `Repudiation`, `Information Disclosure`, `Denial of Service`, `Privilege Escalation`. 

### Threat Risk Ratings

For each threat identified, a risk rating that ranges from *informational* to *critical* is given based on the the likelihood of the threat occurring if a mitigation is not in place and the impact of the threat (i.e. how severely the assets are affected).


| Rating (Score)    | Impact        | Likelihood                        |
| --------------    | -----------   | -------------------------------   |
| Critical (5)      | Extreme impact if exploited. | Threat is almost likely to be exploited. Knowledge of the threat is publicly known. 
| High (4)          | Major impact if exploited. | Threat relatively easy to detect and exploit by an attacker with little skill.| 
| Medium (3)        | Noticeable impact if exploited. | A knowledgeable insider or expert attacker could exploit the threat without much difficulty. | 
| Low (2)           |  Minor impact if exploited. Must be used with other vulnerabilities to be impactful. | Exploiting the threat requires expertise and ressources and can not be easily performed (no predefined method available). |
| Informational (1) | Programming practice or design decision that may not represent an immediate risk, but may have security implications if combined with other threats. | Threat is not likely to be exploit (or atleast on it's own). May be used to gain information for another threat. | 

From the standard risk assessment level documentation, the below table represents the aggregated risk scores calculated by multiplying the impact with the likelihood. 
` risk = impact * likelihood`

| Risk Level &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;   | Risk score &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| ----------    | ----------    |
| Critical      | 20-25         |
| High          | 12-19         |
| Medium        | 6-11          |
| Low           | 2-5           |
| Informational | 1             |


### Threat Assessment
<br />

| ID &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; | 01 &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| --------------------------------------| -----------------------------------------------------------------------|
| Threat                | **Possible dependency zero-day could affect the code and data running inside the BlindAI's enclave.** <br />  A zero-day found in a dependency running inside the secure code could obviously lead to compromise the enclave.  |
| Diagram Element       |   DF1 to DF9    |
| Affected Components   |   TC1, TC3, TC4, TC5    |
| Assets                |   Sensitive Data, User Data, Availability, Model, Code Execution    |
| Threat Agent          |  SecCode     |
| Threat Type           |  `information Disclosure`, `Denial of Service`, `Tampering`    |
| Impact                |  Critical (5)     |
| Likelihood            |  Medium (3)     |
| Total risk rating     |  High (15)     |
| Mitigations           |    For the software dependencies, an update of the TCB and a update of the enclave could resolve the issue as soon as it is updated.<br /> A Zero-day oon the Intel SGX platform  that could only be patched by Intel may lead to some difficulties (Especially if it's hardware related.)  |
| Mitigations implemented?|  For now, the updates are made as soon as they are released.   |

<br />

| ID &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; | 02 &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| --------------------------------------| -----------------------------------------------------------------------|
| Threat                | **An attacker could perform side-channels attacks onto the SGX platform/or against a running vulnerable dependency inside the enclave's code.** <br /> Physical side channels attacks are out of scope. The side-channels could also be related to non-invasive cryptographic & software issues. In that sense, we can give the examples of non constant-time operations that may reveal secrets, or differential fault analysis.       |
| Diagram Element       |   DF9    |
| Affected Components   |   TC1    |
| Assets                |   Sensitive Data, User Data, Model    |
| Threat Agent          |   SecCode    |
| Threat Type           |   `Information Disclosure`, `Privilege Escalation`    |
| Impact                |    Critical (5)   |
| Likelihood            |    Medium (3)   |
| Total risk rating     |    High (15)   |
| Mitigations           |    Applying continuous dynamic and static analysis on the code and its dependencies to detect non constant time operations that could lead to leaking sensitive data. <br />     |
| Mitigations implemented?|  Yes.   |

<br />

| ID &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; | 03 &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| --------------------------------------| -----------------------------------------------------------------------|
| Threat                | **An attacker can try sniffing connection information between the client and the server through the network.**  <br />  An attacker could  try to intercept a TLS connection between the user and the server and try to tamper with it. |
| Diagram Element       |  DF1, DF8, DF9     |
| Affected Components   |  TC1, TC2     |
| Assets                |  Availability, User Data, Model     |
| Threat Agent          |  NSCode    |
| Threat Type           |  `tampering`, `Denial of service`     |
| Impact                |  Low (2)     |
| Likelihood            |  Low (2)     |
| Total risk rating     |  Low (4)     |
| Mitigations           |  Associating BlindAi with a reverse-proxy is necessary on a Production Mode. <br /> Using the remote attestation to verify that the enclave we are running on is valid.     |
| Mitigations implemented?| Partially. Remote attestation implemented and robust and triggers and secure TLS communication with the client   |

<br />

| ID &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; | 04 &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| --------------------------------------| -----------------------------------------------------------------------|
| Threat                |  **An attacker can perform Iago Attacks, to try access the    data or code, against the SGX platform.** <br />  When running on a compromised OS, the TEE applications must run securely even if syscalls and low Privilege operations are compromised too. *Iago attacks* could be related to the I/O or network stack for example.    |
| Diagram Element       |   DF1, DF2    |
| Affected Components   |   TC1, TC3, TC4, TC5    |
| Assets                |   Availability, User data, Model     |
| Threat Agent          |   SecCode    |
| Threat Type           |   `Spoofing`, `Tampering`, `Denial of Service`     |
| Impact                |   High  (4)  |
| Likelihood            |   Low  (2)  |
| Total risk rating     |   Medium (8)   |
| Mitigations           |   The TCB must be as small as possible. The operations with the untrusted part must be kept minimal. <br /> The cryptographic functions and the dependencies for the inference must run inside the enclave and can't be changed directly from the untrusted part.    |
| Mitigations implemented?| Yes, mostly. More research is being done on the network part to see if there is any issue.   |

<br />

| ID &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; | 05 &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;|
| --------------------------------------| -----------------------------------------------------------------------|
| Threat                | **An attacker could perform a membership inference attack.**<br /> BlindAI app uses a modified version of tract as the inference engine. Even though the difficulty to perform such attacks, it could possibly be done in a environment where the data can be easily predicted from the model.   |
| Diagram Element       | DF9      |
| Affected Components   | TC1, TC2     |
| Assets                | User Data, Model      |
| Threat Agent          | NSCode      |
| Threat Type           | `Information Disclosure`      |
| Impact                | High (4)      |
| Likelihood            | Low (2)      |
| Total risk rating     | Medium (8)      |
| Mitigations           | Numerous defenses exists against membership attacks. Like, obfuscating the predictions, or using Differential Privacy-based defenses.      |
| Mitigations implemented?|  No.   |
