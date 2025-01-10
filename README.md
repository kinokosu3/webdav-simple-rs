# WebDAV-RS

一个使用 Rust 实现的 WebDAV 服务器。

## Warning
 - 该项目仅用于学习，不保证稳定性，不保证安全性，不保证性能。
 - this project is only for learning, not for production use.


## 已实现功能

### 基础 HTTP 方法
- [x] OPTIONS
- [x] GET
- [x] PUT
- [x] DELETE
- [ ] HEAD

### WebDAV 特定方法
- [x] PROPFIND
- [x] MKCOL
- [x] COPY
- [x] MOVE
- [ ] LOCK
- [ ] UNLOCK
- [ ] PROPPATCH

## 待实现功能

### 1. 锁定机制 (LOCK/UNLOCK)
- [ ] LOCK 和 UNLOCK 方法
- [ ] 锁定令牌(Lock Token)处理
- [ ] 共享锁和排他锁支持
- [ ] 锁定超时机制

### 2. 属性处理
- [ ] PROPPATCH 方法实现
- [ ] PROPFIND 的深度控制(Depth header)
- [ ] 自定义属性(dead properties)支持
- [ ] 属性的持久化存储

### 3. 条件请求支持
- [ ] If-Match 头处理
- [ ] If-None-Match 头处理
- [ ] If 头(用于锁定验证)支持
- [ ] 条件请求的错误处理

### 4. 安全性
- [ ] 基本身份验证
- [ ] 访问控制列表(ACL)
- [ ] 权限控制系统
- [ ] SSL/TLS 支持

### 5. 其他功能
- [ ] HEAD 方法
- [ ] Overwrite 头处理(COPY/MOVE)
- [ ] Depth: infinity 限制
- [ ] MIME 类型处理
- [ ] 错误处理优化
- [ ] 日志系统

## 优先实现顺序

1. 基本身份验证
2. LOCK/UNLOCK 支持
3. PROPPATCH 方法
4. 条件请求头处理


## 参考资料

- [RFC 4918 - HTTP Extensions for Web Distributed Authoring and Versioning (WebDAV)](https://datatracker.ietf.org/doc/html/rfc4918)
