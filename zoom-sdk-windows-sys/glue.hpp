#pragma once

#include <wtypes.h>
#include <zoom_sdk.h>
#include <auth_service_interface.h>
#include <meeting_service_interface.h>

namespace ZOOMSDK {
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
            if (event.authenticationReturn) {
                event.authenticationReturn(event.callbackData, ret);
            }
        }

        void onLoginRet(LOGINSTATUS ret, IAccountInfo *pAccountInfo) {
            event.loginReturn(event.callbackData, ret, pAccountInfo);
        }

        void onLogout() {}

        void onZoomIdentityExpired() {}

        void onZoomAuthIdentityExpired() {}

        void onLoginReturnWithReason(LOGINSTATUS status,IAccountInfo *account, LoginFailReason reason) {}
    };
}
