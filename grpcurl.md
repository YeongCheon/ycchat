# grpcurl Command List

[grpcurl link](https://github.com/fullstorydev/grpcurl)

## Auth
### SignUp
``` bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/auth/auth.proto \
  -d '{"email": "kyc1682@gmail.com", "username":"test", "password":"test"}' \
  127.0.0.1:50051 \
  ycchat.auth.Auth.SignUp
```

### SignIn
``` bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/auth/auth.proto \
  -d '{"username":"test", "password":"test"}' \
  127.0.0.1:50051 \
  ycchat.auth.Auth.SignIn
```

## Account
### UpdatePassword
```bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/user/user.proto \
  -H authorization:"Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhbnRjaGF0Iiwic3ViIjoiYWNjZXNzX3Rva2VuIiwiYXVkIjoiMDFIQkUzTVlZQkNaQVlLRTlUUzdCWU40TTEiLCJpYXQiOjE2OTU5MTU1MzMsImV4cCI6MTY5NTkxOTEzM30.AwRNTsasOFkeWhk6j4ubqWnB7Y0O7kI7SwCmfdK9H8c" \
  -d '{"current_password": "CURRENT_PASSWORD", "new_password": "NEW_PASSWORD"}' \
  127.0.0.1:50051 \
  ycchat.account.Account.UpdatePassword
```

### DeleteAccount
```bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/user/user.proto \
  -H authorization:"Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhbnRjaGF0Iiwic3ViIjoiYWNjZXNzX3Rva2VuIiwiYXVkIjoiMDFIOVpDV1EyVjNGSlRXUDM5S040OVg4VzYiLCJpYXQiOjE2OTQzNTQ5NzAsImV4cCI6MTY5NDM1ODU3MH0.zFb64v6DBsfk7i2lHuUaWfwNIHU31f00a8JCth53FSg" \
  -d '{"reason": "I no longer use the service."}' \
  127.0.0.1:50051 \
  ycchat.account.Account.UpdatePassword
```

## User
### CreateUser
```bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/user/user.proto \
  -H authorization:"Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhbnRjaGF0Iiwic3ViIjoiYWNjZXNzX3Rva2VuIiwiYXVkIjoiMDFIOVpDV1EyVjNGSlRXUDM5S040OVg4VzYiLCJpYXQiOjE2OTQzNTQ5NzAsImV4cCI6MTY5NDM1ODU3MH0.zFb64v6DBsfk7i2lHuUaWfwNIHU31f00a8JCth53FSg" \
  -d '{"user":{"display_name":"testName","description":"testDescription"}}' \
  127.0.0.1:50051 \
  ycchat.user.User.CreateUser
```

### UpdateUser
```bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/user/user.proto \
  -H authorization:"Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhbnRjaGF0Iiwic3ViIjoiYWNjZXNzX3Rva2VuIiwiYXVkIjoiMDFIOVpDV1EyVjNGSlRXUDM5S040OVg4VzYiLCJpYXQiOjE2OTQzNTQ5NzAsImV4cCI6MTY5NDM1ODU3MH0.zFb64v6DBsfk7i2lHuUaWfwNIHU31f00a8JCth53FSg" \
  -d '{"user":{"name": "users/01H9ZCWQ2V3FJTWP39KN49X8W6","display_name":"testName","description":"testDescription"}}' \
  127.0.0.1:50051 \
  ycchat.user.User.UpdateUser
```

### DeleteUser
```bash
grpcurl \
  --plaintext \
  --import-path ./protobuf \
  --proto protobuf/user/user.proto \
  -H authorization:"Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhbnRjaGF0Iiwic3ViIjoiYWNjZXNzX3Rva2VuIiwiYXVkIjoiMDFIOVpDV1EyVjNGSlRXUDM5S040OVg4VzYiLCJpYXQiOjE2OTQzNTQ5NzAsImV4cCI6MTY5NDM1ODU3MH0.zFb64v6DBsfk7i2lHuUaWfwNIHU31f00a8JCth53FSg" \
  -d '{"name": "users/01H9ZCWQ2V3FJTWP39KN49X8W6"}' \
  127.0.0.1:50051 \
  ycchat.user.User.DeleteUser
```
