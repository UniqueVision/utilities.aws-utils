## Changes

### v0.2.1 (2025/09/13)
* **BREAKING CHANGE**: Removed automatic dummy credential injection from make_client functions
* Fixed authentication issues in ECS/Fargate environments where IAM task roles should be used
* Now properly uses AWS SDK's default credential chain
* Updated README documentation to reflect new authentication behavior

### v0.2.0 (2025/08/14)
* add make_client with timeout

### v0.1.2 (2025/08/08)
* pub use aws_sdk_kinesis

### v0.1.1 (2025/07/11)
* change path

### v0.1.0 (2025/07/10)
* first release