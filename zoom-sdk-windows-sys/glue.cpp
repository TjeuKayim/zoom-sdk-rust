#include "glue.hpp"
#include <iostream>

using namespace ZOOM_SDK_NAMESPACE;

LastErrorType ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorType(const IZoomLastError *self) {
    return self->GetErrorType();
}

UINT64 ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorCode(const IZoomLastError *self) {
    return self->GetErrorCode();
}

const wchar_t *ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorDescription(const IZoomLastError *self) {
    return self->GetErrorDescription();
}

void StringDrop(wchar_t *string) {
    delete string;
}

void ZOOM_SDK_NAMESPACE::AuthServiceEvent_New(AuthServiceEvent *out) {
    new (out) AuthServiceEvent;
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SetEvent(IAuthService *self, IAuthServiceEvent *event) {
    return self->SetEvent(event);
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SDKAuthParam(IAuthService *self, AuthParam param) {
    return self->SDKAuth(param);
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_Login(IAuthService *self, LoginParam param) {
    return self->Login(param);
}

InitParam ZOOM_SDK_NAMESPACE::InitParam_Default() {
    InitParam initParam;
    return initParam;
}

const wchar_t* ZOOM_SDK_NAMESPACE::IAccountInfo_GetDisplayName(IAccountInfo *self) {
    return self->GetDisplayName();
}
LoginType ZOOM_SDK_NAMESPACE::IAccountInfo_GetLoginType(IAccountInfo *self) {
    return self->GetLoginType();
}

SDKError ZOOM_SDK_NAMESPACE::IMeetingsService_HandleZoomWebUriProtocolAction(IMeetingService *self, const wchar_t* protocol_action) {
    return self->HandleZoomWebUriProtocolAction(protocol_action);
}
