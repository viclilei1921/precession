//! MOBI INDX / NCX 索引解析
//!
//! INDX、Tagx、CNCX、NCX 表与 KF8 skeleton/fragment 解析目前内聚在
//! [`super::MobiParser`]（`mod.rs`）中，与正文渲染强耦合。
//! 本模块预留分层位置；后续若抽离索引逻辑，迁入此处即可。
