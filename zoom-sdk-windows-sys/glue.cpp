#include "glue.hpp"
using namespace ZOOM_SDK_NAMESPACE;

LastErrorType ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorType(const IZoomLastError *self) {
    return self->GetErrorType();
}

UINT64 ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorCode(const IZoomLastError *self) {
    return self->GetErrorCode();
}

const wchar_t* ZOOM_SDK_NAMESPACE::IZoomLastError_GetErrorDescription(const IZoomLastError *self) {
    return self->GetErrorDescription();
}

class AuthServiceEvent : public IAuthServiceEvent {
public:
    void (*authenticationReturn)(AuthResult);

    void onAuthenticationReturn(AuthResult ret) {
        authenticationReturn(ret);
    }

    void onLoginRet(LOGINSTATUS ret, IAccountInfo* pAccountInfo) {}

    void onLogout() {}

    void onZoomIdentityExpired() {}

    void onZoomAuthIdentityExpired() {}
};

IAuthServiceEvent* ZOOM_SDK_NAMESPACE::AuthServiceEvent_New(void (*authenticationReturn)(AuthResult)) {
    auto a = new AuthServiceEvent;
    a->authenticationReturn = authenticationReturn;
    return a;
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SetEvent(IAuthService* self, IAuthServiceEvent* event) {
    return self->SetEvent(event);
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SDKAuthParam(ZOOM_SDK_NAMESPACE::IAuthService* self, ZOOM_SDK_NAMESPACE::AuthParam param) {
    return self->SDKAuth(param);
}
