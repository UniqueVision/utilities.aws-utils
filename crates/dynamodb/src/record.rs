use std::collections::HashMap;

use aws_sdk_dynamodb::{
    Client,
    operation::{
        delete_item::DeleteItemOutput, get_item::GetItemOutput, put_item::PutItemOutput,
        update_item::UpdateItemOutput,
    },
    types::{AttributeValue, ReturnValue},
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{Stream, TryStreamExt};

use crate::error::{Error, from_aws_sdk_error};

pub async fn get_item_raw(
    client: &Client,
    table_name: impl Into<String>,
    key: HashMap<String, AttributeValue>,
    consistent_read: Option<bool>,
    expression_attribute_names: Option<HashMap<String, String>>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> Result<GetItemOutput, Error> {
    client
        .get_item()
        .table_name(table_name)
        .set_key(Some(key))
        .set_consistent_read(consistent_read)
        .set_expression_attribute_names(expression_attribute_names)
        .set_projection_expression(projection_expression.map(Into::into))
        .set_attributes_to_get(attributes_to_get.map(|v| v.into_iter().map(Into::into).collect()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_item(
    client: &Client,
    table_name: impl Into<String>,
    key: HashMap<String, AttributeValue>,
    consistent_read: Option<bool>,
    expression_attribute_names: Option<HashMap<String, String>>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> Result<HashMap<String, AttributeValue>, Error> {
    let output = get_item_raw(
        client,
        table_name,
        key,
        consistent_read,
        expression_attribute_names,
        projection_expression,
        attributes_to_get,
    )
    .await?;
    if let Some(item) = output.item {
        Ok(item)
    } else {
        Err(Error::NotFound)
    }
}

pub async fn put_item(
    client: &Client,
    table_name: impl Into<String>,
    item: HashMap<String, AttributeValue>,
    condition_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    return_values: Option<ReturnValue>,
) -> Result<PutItemOutput, Error> {
    client
        .put_item()
        .table_name(table_name)
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_return_values(return_values)
        .set_item(Some(item))
        .set_condition_expression(condition_expression.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_item(
    client: &Client,
    table_name: impl Into<String>,
    key: HashMap<String, AttributeValue>,
    update_expression: impl Into<String>,
    condition_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    return_values: Option<ReturnValue>,
) -> Result<UpdateItemOutput, Error> {
    client
        .update_item()
        .table_name(table_name)
        .set_key(Some(key))
        .set_update_expression(Some(update_expression.into()))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_return_values(return_values)
        .set_condition_expression(condition_expression.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_item(
    client: &Client,
    table_name: impl Into<String>,
    key: HashMap<String, AttributeValue>,
    condition_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    return_values: Option<ReturnValue>,
) -> Result<DeleteItemOutput, Error> {
    client
        .delete_item()
        .table_name(table_name)
        .set_key(Some(key))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_return_values(return_values)
        .set_condition_expression(condition_expression.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub fn scan_stream(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    consistent_read: Option<bool>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> impl Stream<Item = Result<HashMap<String, AttributeValue>, Error>> {
    client
        .scan()
        .table_name(table_name)
        .set_index_name(index_name.map(Into::into))
        .set_filter_expression(filter_expression.map(Into::into))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_consistent_read(consistent_read)
        .set_projection_expression(projection_expression.map(Into::into))
        .set_attributes_to_get(attributes_to_get.map(|v| v.into_iter().map(Into::into).collect()))
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn scan_all(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    consistent_read: Option<bool>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let stream = scan_stream(
        client,
        table_name,
        index_name,
        filter_expression,
        expression_attribute_names,
        expression_attribute_values,
        consistent_read,
        projection_expression,
        attributes_to_get,
    );
    let mut items = Vec::new();
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.try_next().await? {
        items.push(item);
    }
    Ok(items)
}

/// ページネーションなしの単発クエリ。limit で取得件数を制限可能。
#[allow(clippy::too_many_arguments)]
pub async fn query(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    key_condition_expression: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    consistent_read: Option<bool>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
    limit: Option<i32>,
) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let output = client
        .query()
        .table_name(table_name)
        .set_index_name(index_name.map(Into::into))
        .set_key_condition_expression(key_condition_expression.map(Into::into))
        .set_filter_expression(filter_expression.map(Into::into))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_consistent_read(consistent_read)
        .set_projection_expression(projection_expression.map(Into::into))
        .set_attributes_to_get(attributes_to_get.map(|v| v.into_iter().map(Into::into).collect()))
        .set_limit(limit)
        .send()
        .await
        .map_err(from_aws_sdk_error)?;
    // クエリ結果が 0 件の時も正常値を返す
    Ok(output.items.unwrap_or_default()) 
}


#[allow(clippy::too_many_arguments)]
pub fn query_stream(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    key_condition_expression: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    consistent_read: Option<bool>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> impl Stream<Item = Result<HashMap<String, AttributeValue>, Error>> {
    client
        .query()
        .table_name(table_name)
        .set_index_name(index_name.map(Into::into))
        .set_key_condition_expression(key_condition_expression.map(Into::into))
        .set_filter_expression(filter_expression.map(Into::into))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .set_consistent_read(consistent_read)
        .set_projection_expression(projection_expression.map(Into::into))
        .set_attributes_to_get(attributes_to_get.map(|v| v.into_iter().map(Into::into).collect()))
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn query_all(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    key_condition_expression: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    consistent_read: Option<bool>,
    projection_expression: Option<impl Into<String>>,
    attributes_to_get: Option<Vec<impl Into<String>>>,
) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let stream = query_stream(
        client,
        table_name,
        index_name,
        key_condition_expression,
        filter_expression,
        expression_attribute_names,
        expression_attribute_values,
        consistent_read,
        projection_expression,
        attributes_to_get,
    );
    let mut items = Vec::new();
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.try_next().await? {
        items.push(item);
    }
    Ok(items)
}
