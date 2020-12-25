#pragma once

#include <wtypes.h>
#include <zoom_sdk.h>
#include <auth_service_interface.h>

namespace ZOOM_SDK_NAMESPACE {
    extern "C" {
    LastErrorType IZoomLastError_GetErrorType(const IZoomLastError *self);
    UINT64 IZoomLastError_GetErrorCode(const IZoomLastError *self);
    const wchar_t *IZoomLastError_GetErrorDescription(const IZoomLastError *self);
    const size_t IAuthServiceEvent_SIZE = sizeof(IAuthServiceEvent);

    struct AuthServiceEvent {
        void *callbackData;

        void (*authenticationReturn)(void *, AuthResult);
    };
    SDKError IAuthService_SetEvent(IAuthService *self, const AuthServiceEvent *event);
    SDKError IAuthService_SDKAuthParam(IAuthService *self, AuthParam param);
    SDKError IAuthService_Login(IAuthService *self, LoginParam param);
    InitParam InitParam_Default();
    }
}
