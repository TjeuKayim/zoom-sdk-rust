#define _SYSINFOAPI_H_ // conflicts with GetVersion()
#include <wtypes.h>
#include <zoom_sdk.h>
#include <zoom_sdk_ext.h>
#include <premeeting_service_interface.h>
#include <outlook_plugin_integration_helper_interface.h>
#include <calender_service_interface.h>
#include <direct_share_helper_interface.h>
#include <network_connection_handler_interface.h>
#include <auth_service_interface.h>
#include <zoom_sdk_sms_helper_interface.h>
#include <meeting_service_interface.h>
#include <meeting_service_components/meeting_annotation_interface.h>
#include <meeting_service_components/meeting_audio_interface.h>
#include <meeting_service_components/meeting_breakout_rooms_interface.h>
#include <meeting_service_components/meeting_chat_interface.h>
#include <meeting_service_components/meeting_configuration_interface.h>
#include <meeting_service_components/meeting_h323_helper_interface.h>
#include <meeting_service_components/meeting_participants_ctrl_interface.h>
#include <meeting_service_components/meeting_phone_helper_interface.h>
#include <meeting_service_components/meeting_recording_interface.h>
#include <meeting_service_components/meeting_remote_ctrl_interface.h>
#include <meeting_service_components/meeting_sharing_interface.h>
#include <meeting_service_components/meeting_ui_ctrl_interface.h>
#include <meeting_service_components/meeting_video_interface.h>
#include <meeting_service_components/meeting_waiting_room_interface.h>
#include <meeting_service_components/meeting_closedcaption_interface.h>
#include <customized_ui/customized_local_recording.h>
#include <setting_service_interface.h>
#include <customized_ui/customized_ui_mgr.h>
#include <customized_ui/customized_video_container.h>
#include <customized_ui/zoom_customized_ui.h>
#include <customized_ui/customized_share_render.h>
#include <customized_ui/customized_annotation.h>
#include <customized_ui/customized_local_recording.h>

// Virtual Method Wrappers
#include "glue.hpp"
