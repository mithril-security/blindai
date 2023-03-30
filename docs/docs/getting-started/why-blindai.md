# Why BlindAI?
________________

AI models are everywhere. Recent releases, like GPT4, have skyrocketed adoption and democratized their use in all industries. We believe this is great for progress, but that we need to take action to develop solutions that protect confidential data when using those models. Take ChatGPT's example: thousands of employees used it - unrestricted - to help them in their tasks, feeding it company data that could then be viewed by any employee working on improving the next version of the model.

This is why we made BlindAI, so AI models can be used while preserving users' data privacy (and without putting the model at risk). To understand the advantages you'd gain in using us, here's a little tour of the current options used to deploy AI models.

## Don't choose between ease-of-use and privacy
 
AI solutions to be deployed in one of three ways: on the **Cloud**, **on-premise** or **on-device**. 

Cloud deployment offers users **ease-of-use** and a **wide offer of AI models** but it puts users' **data at risk**. Their privacy is left in the hands of the person or company who operates the service, the cloud operator, their sysadmins and other people who each need to be trusted. In general, this means that Cloud deployment is currently not suitable for users with sensitive data such as hospitals wanting to leverage patient data to improve healthcare.

Users with sensitive data may instead turn to on-premise or on-device deployment as a more privacy-friendly alternative to Cloud deployment, but this **increased privacy** comes at a **great cost** in terms of ease-of-use. 
- On-premise deployment requires on-site expertise that is not available to many companies.
- AI models storage space and power are limited by on-site hardware (on-premise deployment) or user devices (on-device deployment).
- AI models need to be embedded in the application distributed to the client, putting them at risk of being stolen in on-device solutions.

**BlindAI** offers the best of both worlds: the **ease-of-use** and **wide offer** of Cloud deployment with the **increased privacy** of on-premise and on-device solutions. We take advantage of the power of confidential computing, and more specifically Intel Software Guard Extension (**Intel SGX**), to enable user data to be processed remotely without the risk of unauthorized access or tampering.

## Using Intel SGX

Intel processors with SGX can create **secure enclaves**. They are self-contained zones where the **processor guarantees** that software running inside **cannot be tampered with** by the host operating system, hypervisor, and even its BIOS. 

Users can upload their data and get their result via a **secure API** provided by the enclave. AI models are run on user data inside the enclave protecting both the user data from potential attacks and providing enhanced security for the AI models compared with on-premise alternatives.

To get introduced to how you can use BlindAI to keep your data safe, check out our [Quick Tour](./quick-tour.ipynb)!

Data security and privacy have become crucial topics with the explosion of information creation and data sharing.

Cloud technology has been critical for this revolution to happen, because it has provided the backbone of many applications powering its growth. Software as a Service (SaaS) companies, hosted on the Cloud, provide fully managed services of great quality, scalability, and availability, under the condition that data is sent to them to handle it.

The problem? In doing so, SaaS providers have also become points of failure in terms of security by handling users' data directly. Compromission of these SaaS providers will induce the leakage of their users' data.

This is why integrating external SaaS providers has become difficult in regulated and sensitive industries such as healthcare, finance or governments, to mitigate these risks.

hospital An example: AI integration for hospitals

To better understand how data can be exposed to SaaS vendors, let’s take a real-life example.

Imagine you have a hospital wanting to use AI to help doctors be more efficient. Doctors often take voice notes in a rush, then need to transcribe them into writing for later reporting. This takes a lot of time and it could be automated with AI speech-to-text.

The hospital doesn’t have the expertise, the infrastructure or the time to do so. If they want to implement it, they would turn to external help - for example an AI provided through the SaaS of a startup. This would be beneficial because there would be no cost for onboarding or for maintenance.

The issue is that, when sending data to that startup, no technical guarantee can be provided that the confidential medical information contained in those audios will not be compromised. Data can be encrypted in transit, and even if the startup uses a secure Cloud, it can still be put at risk because it will be in clear in memory before being analyzed for transcription by the AI model. Therefore the hospital has to entrust their data to this startup which might not have put in place the proper security measures to ensure data confidentiality.

This key security flaw often cuts off this type of collaboration from the get go. The hospital would likely refuse to send their medical data to the SaaS startup, especially if they are at an early stage and don’t have the resources yet to invest in security. Being able to use speech-to-text could have saved many hours for the hospital’s personnel and benefit the startup, but healthcare data security is just too sensitive to take any risk. Now, what if we could allow the startup to provide their service while guaranteeing minimal risk for the hospital?