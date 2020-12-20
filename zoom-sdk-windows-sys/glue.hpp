#pragma once
#include <wtypes.h>
#include <zoom_sdk.h>
#include <auth_service_interface.h>
namespace ZOOM_SDK_NAMESPACE
{
    extern "C" LastErrorType IZoomLastError_GetErrorType(const IZoomLastError *self);
    extern "C" UINT64 IZoomLastError_GetErrorCode(const IZoomLastError *self);
    extern "C" const wchar_t* IZoomLastError_GetErrorDescription(const IZoomLastError *self);
    extern "C" const size_t IAuthServiceEvent_SIZE = sizeof(IAuthServiceEvent);
    extern "C" IAuthServiceEvent* AuthServiceEvent_New(void (*authenticationReturn)(AuthResult));
    extern "C" SDKError IAuthService_SetEvent(IAuthService* self, IAuthServiceEvent* event);
    extern "C" SDKError IAuthService_SDKAuthParam(IAuthService* self, AuthParam param);
}
