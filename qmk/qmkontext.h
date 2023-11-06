#ifndef __QMKONTEXT_H__
#define __QMKONTEXT_H__

#define MAX_QMKONTEXT_COMMANDS 256

typedef bool (*qmkontext_callback_t)(uint8_t);

qmkontext_callback_t qmkontext_callbacks[MAX_QMKONTEXT_COMMANDS];

/**
 * Init function that initializes the callbacks.
 * Must be called on keyboard_post_init_user before registering any callbacks to prevent undefined behaviours when receiving unhandled commands.
 */
void qmkontext_init(void);

/**
 * Method for registering a callback handler.
 * @param event_type The command_id of the qmkontext config.
 * @param callback Callback for handling the event. Should return true if the event has been properly handled.
 */
void qmkontext_register_callback(int event_type, qmkontext_callback_t callback);

/**
 * Method for handling a hid event. The params are the same that raw_hid_receive receives.
 * @param data data pointer received by raw_hid_receive.
 * @param length length indicator raw_hid_receive.
 * @return return value of the qmkontext_callback_t that handles the command. false if no handler has been found.
 */
bool qmkontext_on_receive(uint8_t* data, uint8_t length);

#endif