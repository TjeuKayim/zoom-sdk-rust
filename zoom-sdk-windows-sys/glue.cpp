#include "glue.hpp"
#include <iostream>
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
        std::cout << "onAuthenticationReturn\n";
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
//    a->authenticationReturn(AUTHRET_SERVICE_BUSY);
    return a;
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SetEvent(IAuthService* self, IAuthServiceEvent* event) {
    return self->SetEvent(event);
}

SDKError ZOOM_SDK_NAMESPACE::IAuthService_SDKAuthParam(ZOOM_SDK_NAMESPACE::IAuthService* self, ZOOM_SDK_NAMESPACE::AuthParam param) {
    return self->SDKAuth(param);
}

IAuthServiceEvent* auth_cast() {
    auto a = new AuthServiceEvent;
    return a;
}

void test() {
    std::cout << "hello world from c++\n";
    InitParam initParam;
    initParam.strWebDomain = L"https://zoom.us";
    initParam.strSupportUrl = L"https://zoom.us";

    auto err = InitSDK(initParam);
    if (err != SDKERR_SUCCESS) {
        std::cout << "InitSDK err" << err << "\n";
        return;
    }
    std::cout << "initialized\n";
    IAuthService* authService;
    err = CreateAuthService(&authService);
    if (err != SDKERR_SUCCESS) {
        std::cout << "CreateAuthService err" << err << "\n";
        return;
    }
    std::cout << "authService created\n";
//    auto event = static_cast<IAuthServiceEvent&>(new AuthServiceEvent);
    auto event = auth_cast();
    authService->SetEvent(event);
    AuthParam auth;
    auth.appKey = L"";
    auth.appSecret = L"";
    err = authService->SDKAuth(auth);
    if (err != SDKERR_SUCCESS) {
        std::cout << "SDKAuth err" << err << "\n";
        return;
    }
    std::cout << "finished c++ test\n";
    CleanUPSDK();
}

InitParam ZOOM_SDK_NAMESPACE::InitParam_Default() {
//    test();
    InitParam initParam;
    return initParam;
}
