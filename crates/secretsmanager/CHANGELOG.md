# Changes

## v0.3.0 (2025/09/17)
* **BREAKING CHANGE**: Removed automatic dummy credential injection from make_client functions
* Fixed authentication issues in ECS/Fargate environments where IAM task roles should be used
* Now properly uses AWS SDK's default credential chain
* Updated README documentation to reflect new authentication behavior

## v0.2.0 (2025/08/15)
* add make_client with timeout

## v0.1.0 (2025/07/24)
* first release