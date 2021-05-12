#include "glue.hpp"
#include <iostream>

using namespace ZOOMSDK;

LastErrorType ZOOMSDK::IZoomLastError_GetErrorType(const IZoomLastError *self) {
    return self->GetErrorType();
}

UINT64 ZOOMSDK::IZoomLastError_GetErrorCode(const IZoomLastError *self) {
    return self->GetErrorCode();
}

const wchar_t *ZOOMSDK::IZoomLastError_GetErrorDescription(const IZoomLastError *self) {
    return self->GetErrorDescription();
}

void StringDrop(wchar_t *string) {
    delete string;
}

void ZOOMSDK::AuthServiceEvent_New(AuthServiceEvent *out) {
    new (out) AuthServiceEvent;
}

SDKError ZOOMSDK::IAuthService_SetEvent(IAuthService *self, IAuthServiceEvent *event) {
    return self->SetEvent(event);
}

SDKError ZOOMSDK::IAuthService_SDKAuthParam(IAuthService *self, AuthParam param) {
    return self->SDKAuth(param);
}

SDKError ZOOMSDK::IAuthService_Login(IAuthService *self, LoginParam param) {
    return self->Login(param);
}

InitParam ZOOMSDK::InitParam_Default() {
    InitParam initParam;
    return initParam;
}

const wchar_t* ZOOMSDK::IAccountInfo_GetDisplayName(IAccountInfo *self) {
    return self->GetDisplayName();
}
LoginType ZOOMSDK::IAccountInfo_GetLoginType(IAccountInfo *self) {
    return self->GetLoginType();
}

SDKError ZOOMSDK::IMeetingsService_HandleZoomWebUriProtocolAction(IMeetingService *self, const wchar_t* protocol_action) {
    return self->HandleZoomWebUriProtocolAction(protocol_action);
}
