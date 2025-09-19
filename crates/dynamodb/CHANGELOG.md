# Changes

## v0.4.0 (2025/09/19)
* **BREAKING CHANGE**: query and scan add consistent_read, expression_attribute_names, projection_expression, attributes_to_get

## v0.3.0 (2025/09/17)
* **BREAKING CHANGE**: Removed automatic dummy credential injection from make_client functions
* Fixed authentication issues in ECS/Fargate environments where IAM task roles should be used
* Now properly uses AWS SDK's default credential chain
* Updated README documentation to reflect new authentication behavior

## v0.2.0 (2025/08/14)
* add make_client with timeout

## v0.1.2 (2025/07/31)
* add CacheMap

## v0.1.1 (2025/07/29)
* add is_conditional_check_failed_exception

## v0.1.0 (2025/07/29)
* first release