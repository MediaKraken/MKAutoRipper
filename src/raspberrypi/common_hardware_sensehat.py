from sense_hat import SenseHat


class CommonHardwarePISenseHat:
    """
    Class for interfacing with raspberry pi sensehat
    """

    def __init__(self):
        self.sense_inst = SenseHat()
        self.sense_inst.clear()

    def com_hard_pi_get_temp(self):
        return self.sense_inst.get_temperature()

    def com_hard_pi_get_pressure(self):
        return self.sense_inst.get_pressure()

    def com_hard_pi_get_humidity(self):
        return self.sense_inst.get_humidity()
