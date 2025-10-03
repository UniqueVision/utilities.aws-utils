# Changes

## v0.3.1 (2025/10/03)
* Fix s3 is_exists

## v0.3.0 (2025/09/17)
* **BREAKING CHANGE**: Removed automatic dummy credential injection from make_client functions
* Fixed authentication issues in ECS/Fargate environments where IAM task roles should be used
* Now properly uses AWS SDK's default credential chain
* Updated README documentation to reflect new authentication behavior

## v0.2.1 (2025/08/14)
* Version number correction

## v0.2.0 (2025/08/14)
* add make_client with timeout

## v0.1.0 (2025/07/28)
* first release