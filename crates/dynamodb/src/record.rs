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
) -> Result<GetItemOutput, Error> {
    client
        .get_item()
        .table_name(table_name)
        .set_key(Some(key))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_item(
    client: &Client,
    table_name: impl Into<String>,
    key: HashMap<String, AttributeValue>,
) -> Result<HashMap<String, AttributeValue>, Error> {
    let output = get_item_raw(client, table_name, key).await?;
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

pub fn scan_stream(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
) -> impl Stream<Item = Result<HashMap<String, AttributeValue>, Error>> {
    client
        .scan()
        .table_name(table_name)
        .set_index_name(index_name.map(Into::into))
        .set_filter_expression(filter_expression.map(Into::into))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

pub async fn scan_all(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let stream = scan_stream(
        client,
        table_name,
        index_name,
        filter_expression,
        expression_attribute_names,
        expression_attribute_values,
    );
    let mut items = Vec::new();
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.try_next().await? {
        items.push(item);
    }
    Ok(items)
}

pub fn query_stream(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    key_condition_expression: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
) -> impl Stream<Item = Result<HashMap<String, AttributeValue>, Error>> {
    client
        .query()
        .table_name(table_name)
        .set_index_name(index_name.map(Into::into))
        .set_key_condition_expression(key_condition_expression.map(Into::into))
        .set_filter_expression(filter_expression.map(Into::into))
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

pub async fn query_all(
    client: &Client,
    table_name: impl Into<String>,
    index_name: Option<impl Into<String>>,
    key_condition_expression: Option<impl Into<String>>,
    filter_expression: Option<impl Into<String>>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let stream = query_stream(
        client,
        table_name,
        index_name,
        key_condition_expression,
        filter_expression,
        expression_attribute_names,
        expression_attribute_values,
    );
    let mut items = Vec::new();
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.try_next().await? {
        items.push(item);
    }
    Ok(items)
}
