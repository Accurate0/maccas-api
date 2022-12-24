# maccas-api

## Related
- [Frontend](https://github.com/Accurate0/maccas-web)
- [Infrastructure](https://github.com/Accurate0/infrastructure/tree/main/maccas-api)

A webserver & tasks designed to collate offers (and their redemption) from multiple McDonalds accounts into a REST API.

## Diagram
![Diagram](/resources/diagram.jpg)

## Design

This projects consists of 4 binaries with the rest of the files built as a "maccas" library.

The rocket webserver designed to run on a AWS Lambda or as a standalone binary.
 - api

The following are supplementary tasks executed through AWS SQS on an AWS Lambda:

- cleanup
- images
- refresh

### Refresh

The most important task here is the refresh task, its job is to scrape through all the McDonald's accounts provided using the McDonalds REST API to collect information on the deals present on each of those accounts, these are all stored into DynamoDB.

This task maintains the database structure and ensures its up to date, this the majority a write operation, runs on a schedule with a limit of 50 accounts per run to avoid the ratelimit, run simultaneously across two regions (ap-southeast-1/2) other regions are blocked by McDonalds with the Akamai CDN.

### Images

The images task is run after a complete refresh, provided with the details of the newly added offers, it fetches the images for each offer from the McDonald's CDN and converts, then stores it in an S3 bucket.

### Cleanup

Due to the serverless nature of the API, when a offer is selected and added to the McDonald's account for redemption, it will stay there forever if it is not actually redeemed in store, this causes the account to be "soft-locked".

The cleanup task automatically runs 15 minutes after a offer is selected to remove it from the account if it is still present.
