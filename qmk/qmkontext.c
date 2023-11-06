#include "qmkontext.h"

bool qmkontext_unhandled(uint8_t data) {
    return false;
}

void qmkontext_register_callback(int event_type, qmkontext_callback_t callback) {
    qmkontext_callbacks[event_type] = callback;
}

bool qmkontext_on_receive(uint8_t* data, uint8_t length) {
    uint8_t command = data[0];
    uint8_t payload = data[1];
    return (qmkontext_callbacks[command])(payload);
}


void qmkontext_init(void) {
    for (int i = 0; i < MAX_QMKONTEXT_COMMANDS; i++) {
        qmkontext_register_callback(i, qmkontext_unhandled);
    }
}
