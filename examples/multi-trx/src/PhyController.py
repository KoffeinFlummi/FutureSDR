import requests
import json

class PhyController:

    def __init__(self, url):
        self.url = url
        request = requests.get(url)
        request_json = request.json()
        self.center_offset_mode = True
        self.blocks = request_json["blocks"]
        self.current_phy = 0
        self.rx_freq = [5170e6, 2480e6]
        self.tx_freq = [5170e6, 2480e6]
        self.rx_gain = [60, 50]
        self.tx_gain = [60, 50]
        self.sample_rate = [4e6, 4e6]
        self.channel = [0, 0]
        self.center_freq = [5.1e9, 2.45e9]
        self.rx_freq_offset = [0, 0]
        self.tx_freq_offset = [0, 0]

        # get id of relevant blocks
        source_selector_id = -1 
        sink_selector_id = -1
        message_selector_id = -1
        soapy_source_id = -1
        soapy_sink_id = -1

        for block in self.blocks:
            if block["instance_name"] == "Selector<2, 1>_0":
                source_selector_id = block["id"]
            if block["instance_name"] == "Selector<1, 2>_0":
                sink_selector_id = block["id"]
            if block["instance_name"] == "MessageSelector_0":
                message_selector_id = block["id"]
            if block["instance_name"] == "SoapySink_0":
                soapy_sink_id = block["id"]
            if block["instance_name"] == "SoapySource_0":
                soapy_source_id = block["id"]

        if (source_selector_id == -1) or (sink_selector_id == -1) or (soapy_source_id == -1) or (soapy_sink_id == -1) or (message_selector_id == -1): 
            if source_selector_id == -1:
                print("Cannot find source selector!")
            if sink_selector_id == -1:
                print("Cannot find sink selector!")
            if soapy_source_id == -1:
                print("Cannot find soapy source")
            if soapy_sink_id == -1:
                print("Cannot find soapy sink")
            if message_selector_id == -1:
                print("Cannot find message selector")


        self.source_selector_url = "{0}block/{1}/call/0/".format(url, source_selector_id)
        self.sink_selector_url = "{0}block/{1}/call/1/".format(url, sink_selector_id)
        self.message_selector_url = "{0}block/{1}/call/1/".format(url, message_selector_id)
        self.soapy_source_freq_url = "{0}block/{1}/call/0/".format(url, soapy_source_id)
        self.soapy_source_gain_url = "{0}block/{1}/call/1/".format(url, soapy_source_id)
        self.soapy_source_sample_rate_url = "{0}block/{1}/call/2/".format(url, soapy_source_id)
        self.soapy_source_center_freq_url = "{0}block/{1}/call/4/".format(url, soapy_source_id)
        self.soapy_source_freq_offset_url = "{0}block/{1}/call/5/".format(url, soapy_source_id)
        self.soapy_sink_freq_url = "{0}block/{1}/call/0/".format(url, soapy_sink_id)
        self.soapy_sink_gain_url = "{0}block/{1}/call/1/".format(url, soapy_sink_id)
        self.soapy_sink_sample_rate_url = "{0}block/{1}/call/2/".format(url, soapy_sink_id)
        self.soapy_sink_center_freq_url = "{0}block/{1}/call/4/".format(url, soapy_sink_id)
        self.soapy_sink_freq_offset_url = "{0}block/{1}/call/5/".format(url, soapy_sink_id)

    #determines if center frequency and offset frequency is used to tune the sdr or a single frequency
    def use_center_frequency_offset_mode(self, bool):
        self.center_offset_mode = bool

    #sets rx frequency  of the corresponding phy. applies config on next phy selection
    def set_rx_frequency_config(self, phy, frequency):
        self.rx_freq[phy] = frequency
    
    #sets tx frequency  of the corresponding phy. applies config on next phy selection
    def set_tx_frequency_config(self, phy, frequency):
        self.tx_freq[phy] = frequency

    #set rx gain  of the corresponding phy. applies config on next phy selection
    def set_rx_gain_config(self, phy, gain):
        self.rx_gain[phy] = gain

    #set tx gain  of the corresponding phy. applies config on next phy selection
    def set_tx_gain_config(self, phy, gain):
        self.tx_gain[phy] = gain

    #sets sample rate of the corresponding phy. applies config on next phy selection
    def set_sample_rate_config(self, phy, sample_rate):
        self.sample_rate[phy] = sample_rate

    #sets channel of receiver/transmitter. applies config on next phy selection
    def set_sample_channel_config(self, receiver, transmitter):
        self.channel = [receiver, transmitter]

    #set center frquency of the corresponding phy. applies config on next phy selection
    def set_center_frequency_config(self, phy, freq):
        self.center_freq[phy] = freq

    #set frequency offset of the corresponding phy. applies config on next phy selection
    def set_rx_frequency_offset_config(self ,phy, offset):
        self.rx_freq_offset[phy] = offset

    #set frequency offset of the corresponding phy. applies config on next phy selection
    def set_tx_frequency_offset_config(self ,phy, offset):
        self.tx_freq_offset[phy] = offset



    #sets rx frequency via message handler
    def set_rx_frequency(self, frequency):
        requests.post(self.soapy_source_frequency_url, json = {"F64" : int(frequency)})
    
    #sets tx frequency via message handler
    def set_tx_frequencyü(self,frequency):
        requests.post(self.soapy_sink_frequency_url, json = {"F64" : int(frequency)})

    #set rx gain via message handler
    def set_rx_gainü(self, frequency):
        requests.post(self.soapy_source_gain_url, json = {"F64" : int(gain)})

    #set tx gain via message handler
    def set_tx_gainü(self, gain):
        requests.post(self.soapy_sink_gain_url, json = {"F64" : int(gain)})

    #sets sample rate via message handler
    def set_rx_sample_rate(self, sample_rate):
        requests.post(self.soapy_source_sample_rate_url, json = {"F64" : int(sample_rate)})

    #sets sample rate via message handler
    def set_tx_sample_rate(self, sample_rate):
        requests.post(self.soapy_sink_sample_rate_url, json = {"F64" : int(sample_rate)})

    #set center frquency via message handler
    def set_rx_center_frequency(self, freq, channel):
        requests.post(self.soapy_source_center_freq_url, json = {"VecPmt":[{"F64":int(freq)},{"U32":int(channel)}]})

    #set center frquency via message handler
    def set_tx_center_frequency(self, freq, channel):
        requests.post(self.soapy_sink_center_freq_url, json = {"VecPmt":[{"F64":int(freq)},{"U32":int(channel)}]})

    #set frequency offset via message handler
    def set_rx_frequency_offset(self, offset, channel):
        requests.post(self.soapy_source_freq_offset_url, json = {"VecPmt":[{"F64":int(offset)},{"U32":int(channel)}]})

    #set frequency offset via message handler
    def set_tx_frequency_offset(self, offset, channel):
        requests.post(self.soapy_sink_center_freq_url, json = {"VecPmt":[{"F64":int(offset)},{"U32":int(channel)}]})


    #select the PHY protocol (WLAN = 0, Bluetooth =1)
    def select_phy(self, phy):
        requests.post(self.source_selector_url, json = {"U32" : phy})
        requests.post(self.sink_selector_url, json = {"U32" : phy})
        requests.post(self.message_selector_url, json = {"U32" : phy})
        requests.post(self.soapy_source_gain_url, json = {"F64" : int(self.rx_gain[phy])})
        requests.post(self.soapy_source_sample_rate_url, json = {"F64" : int(self.sample_rate[phy])})
        requests.post(self.soapy_sink_gain_url, json = {"F64" : int(self.tx_gain[phy])})
        requests.post(self.soapy_sink_sample_rate_url, json = {"F64" : int(self.sample_rate[phy])})
        if self.center_offset_mode:
            requests.post(self.soapy_source_center_freq_url, json = {"VecPmt":[{"F64":int(self.center_freq[phy])},{"U32":self.channel[0]}]})
            requests.post(self.soapy_sink_center_freq_url, json = {"VecPmt":[{"F64":int(self.center_freq[phy])},{"U32":self.channel[1]}]})
            requests.post(self.soapy_source_freq_offset_url, json = {"VecPmt":[{"F64":int(self.rx_freq_offset[phy])},{"U32":self.channel[0]}]})
            requests.post(self.soapy_sink_freq_offset_url, json = {"VecPmt":[{"F64":int(self.tx_freq_offset[phy])},{"U32":self.channel[1]}]})
        else:
            requests.post(self.soapy_source_freq_url, json = {"F64" : int(self.rx_freq[phy])})
            requests.post(self.soapy_sink_freq_url, json = {"F64" : int(self.tx_freq[phy])})
        self.current_phy = phy

    #switches to the other phy 
    def switch_phy(self):
        if self.current_phy == 0:
            self.current_phy = 1
        else:
            self.current_phy = 0
        requests.post(self.source_selector_url, json = {"U32" : self.current_phy})
        requests.post(self.sink_selector_url, json = {"U32" : self.current_phy})
        requests.post(self.message_selector_url, json = {"U32" : self.current_phy})
        requests.post(self.soapy_source_gain_url, json = {"F64" : int(self.rx_gain[self.current_phy])})
        requests.post(self.soapy_source_sample_rate_url, json = {"F64" : int(self.sample_rate[self.current_phy])})
        requests.post(self.soapy_sink_gain_url, json = {"F64" : int(self.tx_gain[self.current_phy])})
        requests.post(self.soapy_sink_sample_rate_url, json = {"F64" : int(self.sample_rate[self.current_phy])})
        if self.center_offset_mode:
            requests.post(self.soapy_source_center_freq_url, json = {"VecPmt":[{"F64":int(self.center_freq[self.current_phy])},{"U32":self.channel[0]}]})
            requests.post(self.soapy_sink_center_freq_url, json = {"VecPmt":[{"F64":int(self.center_freq[self.current_phy])},{"U32":self.channel[1]}]})
            requests.post(self.soapy_source_freq_offset_url, json = {"VecPmt":[{"F64":int(self.rx_freq_offset[self.current_phy])},{"U32":self.channel[0]}]})
            requests.post(self.soapy_sink_freq_offset_url, json = {"VecPmt":[{"F64":int(self.tx_freq_offset[self.current_phy])},{"U32":self.channel[1]}]})
        else:
            requests.post(self.soapy_source_freq_url, json = {"F64" : int(self.rx_freq[self.current_phy])})
            requests.post(self.soapy_sink_freq_url, json = {"F64" : int(self.tx_freq[self.current_phy])})




    
