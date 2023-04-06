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

## Next steps 

To get see a hands-on demo of how you can use the BlindAI API, check out our [Quick Tour](./quick-tour.ipynb)!

To find out more about the Confidential technologies behind BlindAI, check out our [guide to Confidential Computing](confidential_computing.md).
