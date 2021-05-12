#pragma once

#include <wtypes.h>
#include <zoom_sdk.h>
#include <auth_service_interface.h>
#include <meeting_service_interface.h>

namespace ZOOMSDK {
    InitParam InitParam_Default();
    LastErrorType IZoomLastError_GetErrorType(const IZoomLastError *self);
    UINT64 IZoomLastError_GetErrorCode(const IZoomLastError *self);
    const wchar_t *IZoomLastError_GetErrorDescription(const IZoomLastError *self);
    void StringDrop(wchar_t *string);

    struct CAuthServiceEvent {
        void *callbackData;

        void (*authenticationReturn)(void *, AuthResult);
        void (*loginReturn)(void *, LOGINSTATUS, IAccountInfo*);
    };
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

        void onLoginReturnWithReason(LOGINSTATUS status,IAccountInfo *account, LoginFailReason reason) {}
    };
    void AuthServiceEvent_New(AuthServiceEvent *out);
    SDKError IAuthService_SetEvent(IAuthService *self, IAuthServiceEvent *event);
    SDKError IAuthService_SDKAuthParam(IAuthService *self, AuthParam param);
    SDKError IAuthService_Login(IAuthService *self, LoginParam param);

    const wchar_t* IAccountInfo_GetDisplayName(IAccountInfo *self);
    LoginType IAccountInfo_GetLoginType(IAccountInfo *self);

    SDKError IMeetingsService_HandleZoomWebUriProtocolAction(IMeetingService *self, const wchar_t* protocol_action);
}
