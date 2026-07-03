typedef enum {
    RADIO_RTL8822BS,
    RADIO_RTL8822CS,
    RADIO_RTL8733BS,
    RADIO_BCM4335,
	RADIO_BCM4354,
	RADIO_BCM4358,
	RADIO_BCM43569,
	RADIO_SD8997,
	RADIO_SD8987,
	RADIO_IW416,
    RADIO_UNKNOWN
} Radio;

const char* radio_to_string(Radio radio) {
    switch (radio) {
        case RADIO_RTL8822BS:  return "RTL8822BS";
        case RADIO_RTL8822CS:  return "RTL8822CS";
        case RADIO_RTL8733BS:  return "RTL8733BS";
        case RADIO_BCM4335:    return "BCM4335";
        case RADIO_BCM4354:    return "BCM4354";
        case RADIO_BCM4358:    return "BCM4358";
        case RADIO_BCM43569:   return "BCM43569";
        case RADIO_SD8997:     return "SD8997";
        case RADIO_SD8987:     return "SD8987";
        case RADIO_IW416:      return "IW416";
        default:               return "UNKNOWN";
    }
}

Radio radio_from_pid(unsigned int pid) {
    switch (pid) {
        case 0xb822: return RADIO_RTL8822BS;
        case 0xc822: return RADIO_RTL8822CS;
        case 0xb733: return RADIO_RTL8733BS;
        case 0x4335: return RADIO_BCM4335;
        case 0x4354: return RADIO_BCM4354;
        case 0x4358: return RADIO_BCM4358;
        case 0xaa31: return RADIO_BCM43569;
        case 0x9141: return RADIO_SD8997;
        case 0x9149: return RADIO_SD8987;
        case 0x9159: return RADIO_IW416;
        default:     return RADIO_UNKNOWN; 
    }
}
