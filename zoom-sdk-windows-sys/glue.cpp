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

class AuthServiceEvent : public IAuthServiceEvent {
public:
    CAuthServiceEvent event;

    void onAuthenticationReturn(AuthResult ret) {
        event.authenticationReturn(event.callbackData, ret);
    }

    void onLoginRet(LOGINSTATUS ret, IAccountInfo *pAccountInfo) {
        event.loginReturn(event.callbackData, ret, pAccountInfo);
    }

    void onLogout() {}

    void onZoomIdentityExpired() {}

    void onZoomAuthIdentityExpired() {}
};

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SetEvent(IAuthService *self, const CAuthServiceEvent *event) {
    if (!event->authenticationReturn || !event->loginReturn) {
        return SDKERR_INVALID_PARAMETER;
    }
    auto wrap = new AuthServiceEvent; // TODO: free memory
    wrap->event = *event;
    return self->SetEvent(wrap);
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

const wchar_t* IAccountInfo_GetDisplayName(IAccountInfo *self) {
    return self->GetDisplayName();
}
LoginType IAccountInfo_GetLoginType(IAccountInfo *self) {
    return self->GetLoginType();
}
void IAccountInfo_Drop(IAccountInfo *self) {
    delete self;
}
