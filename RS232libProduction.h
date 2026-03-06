#pragma once

/* @file
*  @brief Interface for test commands.
*
*  Version: 4.1
*  Created on: 20 November 2018
*  Author: Tobias Bertilsson, Jonas Ahnstrom, Lucas Magnusson Globe Group
*/

#include <windows.h>
#include <stdint.h>

/*################################################################################
 *
 * Production test: Parameters/commands in the CommDll library.
 *
 *  # Possible values of "ReturnCode" for ParamId functions:
 *	0		OK
 *	1		Error, invalid data
 *	2		Error, unknown
 *	3		Error, not available
 *	254		Exception error from CommDLL
 *	255		NAK response
 *
 * # Example use of Param functions:
 *
 * {
 *		uint8_t ReturnCode = 9;		// ReturnCode should be initialized to a value the CommDll library don't return, in this case 9.
 *		uint32_t PairingCode;
 *
 *		ParamId300(&ReturnCode, &PairingCode);
 *
 *		if (0 == ReturnCode)		// Check that data is recieved and valid.
 *		{
 *			printf("PairingCode: %u", PairingCode);
 *		}
 *		else if (3 == ReturnCode)	// Otherwise there was a problem somewhere.
 *		{
 *			printf("Not available.");
 *		}
 *		else						// RetunCode is 2, 254 or 255. Since ParamId300 is a get function, return value 1 is not applicable.
 *		{
 *			printf("Something else went wrong.");
 *		}
 * }
 *
 * # For more information about each Param see the IRS documentation.
 *
 *################################################################################*/

#ifdef _COM_BUILD_PRODUCTION_IRS_DLL

typedef unsigned char           u8;
typedef unsigned short          u16;
typedef unsigned int            u32;
typedef unsigned long long      u64;
typedef signed char             s8;
typedef short                   s16;
typedef int                     s32;
typedef long long               s64;

/* Required for opal-api.h */
typedef u8  uint8_t;
typedef u16 uint16_t;
typedef u32 uint32_t;
typedef u64 uint64_t;
typedef s8  int8_t;
typedef s16 int16_t;
typedef s32 int32_t;
typedef s64 int64_t;

#endif _COM_BUILD_PRODUCTION_IRS_DLL

void InitLog();
void ShutdownLog();

/*
Function name: EnableLog.
Description: Enables logging of transmissions to log file. On by default.
Parameters: uint8_t Log, which log file.
 0 = Enable byte log.
*/
void EnableLog(uint8_t Log);

/*
Function name: DisableLog.
Description: Disables logging.
Parameters: uint8_t Log, which log file.
 0 = Disable byte log.
*/
void DisableLog(uint8_t Log);

/*
Function name: EnableManchesterCode.
Description: Set Manchestercoding.
Parameters: Bool state, true or false if it should be manchestercoding.
*/
void EnableManchesterCode(bool State);



uint16_t Connect(uint8_t Receiver, uint8_t Sender, uint16_t PortNr, uint8_t MsgType = 16);


/*
For all connect functions.
Parameters: PortNr, COM port number.
Return:
	0 = OK.
	1 = Connection failed.
	2 = Could not open COM port.
*/

/*Connect to mower, UART interface (OPI-connector)*/
uint8_t ConnectMower(uint16_t PortNr);
/*Connect to mower, network interface (TCP/IP)*/
uint8_t ConnectMowerViaNetwork(char* IPAddress, char* PortNr);
/*Connect to mower and send CS tunnelling command*/
uint8_t ConnectMowerViaCS(uint16_t PortNr);

/*Connect to mower via CS connector (Nozzle)*/
uint8_t ConnectMowerViaCharging(uint16_t PortNr);

/*Connect to mower via CS connector (Nozzle), without manchester coding*/
uint8_t ConnectMowerViaChargingNoManchester(uint16_t PortNr);

/*Connect to charging station*/
uint8_t ConnectCS(uint16_t PortNr);

/*Connect to charging station via mower UART interface (OPI-connector)*/
uint8_t ConnectCSViaMower(uint16_t PortNr);

/*
Function name: CloseCOMPort
Description: Closes the Comport.
Return:
	0 if OK
	1 if error
*/
uint8_t CloseCOMPort();

void SetReadTimeout(uint32_t Timeout_ms);
void SetPayloadProtocolId(uint8_t Id, uint8_t* Key);

/*Set DNS name*/
void ParamId018(uint8_t* ReturnCode, uint16_t PinCode, uint8_t* DNSNameBuff);

/*Get DNS name*/
void ParamId020(uint8_t* ReturnCode, uint16_t PinCode, uint8_t* DNSNameBuff);

/*Set DNS Port*/
void ParamId022(uint8_t* ReturnCode, uint16_t PinCode, uint16_t DnsPort);

/*Get DNS Port*/
void ParamId024(uint8_t* ReturnCode, uint16_t PinCode, uint16_t* DnsPort);

/*Set Mower production information*/
void ParamId058(uint8_t* ReturnCode, uint32_t MoverPN, uint16_t MoverRev, uint32_t MoverProTime, uint8_t* MowerNameBuff);

/*Get Mower production information*/
void ParamId060(uint8_t* ReturnCode, uint32_t* MowerPN, uint16_t* MoverRev, uint32_t* MowrProTime, uint8_t* MowerNameBuff, uint16_t* MowerDevGrNo, uint8_t* MowerDevSubNo, uint8_t* MowerDevVarNo, uint32_t* MowerMainBoaSerNo);

/*Set Mower Device Type*/
void ParamId062(uint8_t* ReturnCode, uint16_t MowerDevGrNo, uint8_t MowerDevSubNo, uint8_t MowerDevVarNo);

/*Get Mower HW information*/
#if (PLATFORM_VERSION == 1)
void ParamId064(uint8_t* ReturnCode, uint16_t* MowerDevGrNo, uint8_t* MowerDevSubNo, uint8_t* MowerDevVarNo,
	uint16_t* BodyDevGrNo, uint8_t* BodyDevSubNo, uint8_t* inBodyDevVarNo,
	uint16_t* ChassisDevGrNo, uint8_t* ChassisDevSubNo, uint8_t* inChassisDevVarNo,
	uint16_t* WheelMotorDevGrNo, uint8_t* WheelMotorDevSubNo, uint8_t* WheelMotorDevVarNo,
	uint16_t* CuttingMotorDevGrNo, uint8_t* CuttingMotorDevSubNo, uint8_t* CuttingMotorDevVarNo,
	uint16_t* BatteryDevGrNo, uint8_t* BatteryDevSubNo, uint8_t* BatteryDevVarNo, uint8_t* NrOfBattery,
	uint16_t* LiftSensorDevGrNo, uint8_t* LiftSensorDevSubNo, uint8_t* LiftSensorDevVarNo, uint8_t* NrOfliftSensor,
	uint16_t* ColliSensorDevGrNo, uint8_t* ColliSensorDevSubNo, uint8_t* ColliSensorDevVarNo, uint8_t* NrOfColliSensor,
	uint16_t* LoopSensorDevGrNo, uint8_t* LoopSensorDevSubNo, uint8_t* LoopSensorDevVarNo, uint8_t* NrOfRearLoopSensor, uint8_t* NrOffrontLoopSensor,
	uint16_t* ComBoardDevGrNo, uint8_t* ComBoardDevSubNo, uint8_t* ComBoardDevVarNo,
	uint16_t* ChargStationDevGrNo, uint8_t* ChargStationDevSubNo, uint8_t* ChargStationDevVarNo,
	uint16_t* HmiBoardDevGrNo, uint8_t* HmiBoardDevSubNo, uint8_t* HmiBoardDevVarNo,
	uint16_t* DisablingDeviceBoardDevGrNo, uint8_t* DisablingDeviceBoardDevSubNo, uint8_t* DisablingDeviceBoardDevVarNo);
#else 
void ParamId064(uint8_t* ReturnCode, uint16_t* MowerDevGrNo, uint8_t* MowerDevSubNo, uint8_t* MowerDevVarNo,
	uint16_t* BodyDevGrNo, uint8_t* BodyDevSubNo, uint8_t* inBodyDevVarNo,
	uint16_t* ChassisDevGrNo, uint8_t* ChassisDevSubNo, uint8_t* inChassisDevVarNo,
	uint16_t* WheelMotorDevGrNo, uint8_t* WheelMotorDevSubNo, uint8_t* WheelMotorDevVarNo,
	uint16_t* CuttingMotorDevGrNo, uint8_t* CuttingMotorDevSubNo, uint8_t* CuttingMotorDevVarNo,
	uint16_t* BatteryDevGrNo, uint8_t* BatteryDevSubNo, uint8_t* BatteryDevVarNo, uint8_t* NrOfBattery,
	uint16_t* LiftSensorDevGrNo, uint8_t* LiftSensorDevSubNo, uint8_t* LiftSensorDevVarNo, uint8_t* NrOfliftSensor,
	uint16_t* ColliSensorDevGrNo, uint8_t* ColliSensorDevSubNo, uint8_t* ColliSensorDevVarNo, uint8_t* NrOfColliSensor,
	uint16_t* LoopSensorDevGrNo, uint8_t* LoopSensorDevSubNo, uint8_t* LoopSensorDevVarNo, uint8_t* NrOfRearLoopSensor, uint8_t* NrOffrontLoopSensor,
	uint16_t* ComBoardDevGrNo, uint8_t* ComBoardDevSubNo, uint8_t* ComBoardDevVarNo,
	uint16_t* ChargStationDevGrNo, uint8_t* ChargStationDevSubNo, uint8_t* ChargStationDevVarNo,
	uint16_t* HmiBoardDevGrNo, uint8_t* HmiBoardDevSubNo, uint8_t* HmiBoardDevVarNo,
	uint16_t* DisablingDeviceBoardDevGrNo, uint8_t* DisablingDeviceBoardDevSubNo, uint8_t* DisablingDeviceBoardDevVarNo,
	uint16_t* heightAdjDevGrNo, uint8_t* heightAdjDevSubNo, uint8_t* heightAdjDevVarNo,
	uint16_t* heightSensAdjDevGrNo, uint8_t* heightSensAdjDevSubNo, uint8_t* heightSensAdjDevVarNo,
	uint16_t* radarBoardDevGrNo, uint8_t* radarBoardDevSubNo, uint8_t* radarBoardDevVarNo,
	uint16_t* rearLightDevGrNo, uint8_t* rearLightDevSubNo, uint8_t* rearLightDevVarNo,
	uint16_t* frontLightDevGrNo, uint8_t* frontLightDevSubNo, uint8_t* frontLightDevVarNo,
	uint16_t* appBoardDevGrNo, uint8_t* appBoardDevSubNo, uint8_t* appBoardDevVarNo,
	uint16_t* cameraBoardDevGrNo, uint8_t* cameraBoardDevSubNo, uint8_t* cameraBoardDevVarNo);
#endif

/*Get Mower Recover Main Application SW*/
void ParamId066(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Mower Main Application SW*/
void ParamId068(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Mower Cutting Application SW*/
void ParamId070(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Mower Main Boot SW*/
void ParamId072(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Mower Cutting Boot SW*/
void ParamId074(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get CS Application SW*/
#if (PLATFORM_VERSION == 1)
void ParamId076(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);
#else
void ParamId076(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo, uint32_t* SerialNo, uint32_t TargetSerialNo);
#endif

/*Get CS Boot SW*/
#if (PLATFORM_VERSION == 1)
void ParamId078(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);
#else
void ParamId078(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo, uint32_t* SerialNo, uint32_t TargetSerialNo);
#endif

/*Get mower status*/
void ParamId080(uint8_t* ReturnCode, uint8_t* MowerMainP, uint8_t* MowerSubState, uint32_t* TimeStpNxtStart, uint8_t* BattStat, uint16_t* StatFlags, uint8_t* WrlessConStat, uint8_t* SignQuality, uint8_t* SourceForNextStartStop, uint16_t* Notify, uint8_t* ConfigurationHash);

/*Get GNSS position*/
void ParamId094(uint8_t* ReturnCode, int32_t* latDeg, int32_t* longDeg, uint8_t* HDOP);

/*Get wireless communication status*/
void ParamId096(uint8_t* ReturnCode, uint8_t* GprsLteStat, uint8_t* GprsLteSignQual, uint8_t* GNSSHWstat, uint8_t* SimStatus, uint8_t* BLEHwStat, uint8_t* GprsLteConnStat, uint8_t* BLEConnStat, uint8_t* WiFiConnStat, uint8_t* WiFiHwStat, uint8_t* LoraConnStat, uint8_t* LoraHwStat, uint8_t* RtkHwStat, uint8_t* RtkConnStat, uint32_t* ConnectedRaSerial);


/*Get real time data for battery 1*/
void ParamId108(uint8_t* ReturnCode, uint16_t* BattVolMW, int16_t* BattCurr, int16_t* BattEnLvl, int16_t* BattTemp, uint16_t* MainVoltage);

/*Set Main Mcu serial number*/
void ParamId110(uint8_t* ReturnCode, uint8_t* Serialbuf);

/*Set MCU2 serial number */
void ParamId112(uint8_t* ReturnCode, uint8_t* Serialbuf);

/*Get real time data for wheel motor*/
void ParamId114(uint8_t* ReturnCode, int8_t* RightWhlMotorP, int16_t* RightWhlMotorCurr, int16_t* RightWhlMotorSp, int8_t* LeftWhlMotorP, int16_t* LefWhlMotorCurr, int16_t* LefWhlMotorSp);

/*Get real time data for cutting motor*/
void ParamId116(uint8_t* ReturnCode, int8_t* CuttingMotorP, int16_t* CuttingMotorCurr, int16_t* CuttingMotorSpeed);

/*Get Mower basic sensor data*/
void ParamId118(uint8_t* ReturnCode, uint8_t* CollisionSen, uint8_t* LiftSen, uint16_t* StatusFlags, uint8_t* stopSen, uint8_t* disablingSen);

/*Get Accelerometer data*/
void ParamId120(uint8_t* ReturnCode, int16_t* PitchAngle, int16_t* RollAngle, int16_t* MoverVertGForce, float* accel_x, float* accel_y, float* accel_z, float* gyro_x, float* gyro_y, float* gyro_z, uint32_t* utc_sec, uint32_t* utc_usec);

/*Get Temperature*/
void ParamId122(uint8_t* ReturnCode, int16_t* MoverTemp, int16_t* CSTemp, int16_t* BatTemp, int16_t* CuttingMotorTemp, int16_t* RightWmTemp, int16_t* LeftWmTemp, int16_t* AppBoardTemp, int16_t* RadarTemp);

/*Get Public event log*/
void ParamId132(uint8_t* ReturnCode, uint8_t Index, uint8_t NrEvent, uint8_t* SeverityBuf, uint16_t* EventidBuf, uint32_t* TimeStampBuf, uint8_t* PrevMowerMStatBuf, uint8_t* MowerSubStateBuf, int8_t* PitchAngleBuf, int8_t* RollAngleBuf, uint32_t* LogDataOne, uint32_t* LogDataTwo);

/*Get Internal event log*/
void ParamId134(uint8_t* ReturnCode, uint8_t Index, uint8_t NrEvent, uint8_t* SeverityBuf, uint16_t* EventidBuf, uint32_t* TimeStampBuf, uint8_t* PrevMowerMStatBuf, uint8_t* MowerSubStateBuf, int8_t* PitchAngleBuf, int8_t* RollAngleBuf, uint32_t* LogDataOneBuf, uint32_t* LogDataTwoBuf);

/*Set Clock*/
void ParamId154(uint8_t* ReturnCode, uint32_t UTCTimestamp, int32_t TimeZoneOffset);

/*Get Clock*/
void ParamId156(uint8_t* ReturnCode, uint32_t* UTCTimestamp, int32_t* TimeZoneOffset);

/*Pause (stop) mower*/
void ParamId226(uint8_t* ReturnCode, uint16_t None);

/*Tilt sensor calibration to 0 deg*/
void ParamId232(uint8_t* ReturnCode, uint16_t None);

/*Switch to Power off mode*/
void ParamId234(uint8_t* ReturnCode, uint16_t None);

/*Reset "Cutting time list"*/
void ParamId238(uint8_t* ReturnCode, uint16_t None);

/*Reset "Public event log"*/
void ParamId242(uint8_t* ReturnCode, uint16_t None);

/*Reset "Internal event log"*/
void ParamId244(uint8_t* ReturnCode, uint16_t None);

/*Reset "Statistic counters"*/
void ParamId246(uint8_t* ReturnCode, uint16_t None);

/*CS Cutting motor test*/
void ParamId252(uint8_t* ReturnCode, int16_t CuttingMotor);

/*Right wheel motor power*/
void ParamId254(uint8_t* ReturnCode, int16_t RightMotorSpeed);

/*Left wheel motor power*/
void ParamId256(uint8_t* ReturnCode, int16_t LeftMotorSpeed);

/*LED test*/
void ParamId260(uint8_t* ReturnCode, uint8_t LEDByte);

/*Sound test*/
void ParamId262(uint8_t* ReturnCode, uint16_t SoundTest);

/*Get MCU serial number*/
void ParamId268(uint8_t* ReturnCode, uint8_t* Serialbuf);

/*Get Battery information*/
void ParamId272(uint8_t* ReturnCode, uint32_t* BattPackPN, uint16_t* BattPackRev, uint32_t* BattPackProdDate, uint32_t* BattSwVer,
	uint32_t* BattSerNo, uint32_t* BattDevGrNo, uint32_t* BattSubDevNo, uint16_t* BattVarNo,
	uint16_t* BmsDevGrNo, uint16_t* BmsSubDevNo, uint16_t* BmsVarNo, uint32_t* BmsPcbaPN, uint16_t* BmsPcbaRev, uint32_t* BmsTempSensorType);

/*Get information about cellular communication*/
void ParamId276(uint8_t* ReturnCode, uint8_t* IMEIBuff, uint8_t* SimIMEIBuff, uint8_t* SimIccidBuff, uint8_t* OperNameBuff);

/*Get Pairing code*/
void ParamId300(uint8_t* ReturnCode, uint32_t* PairingCode);

/*Forced SW download*/
void ParamId306(uint8_t* ReturnCode, uint8_t SubMsg, uint16_t DevGroupNo, uint8_t SubGroupNo, uint8_t VarNo, uint32_t RelAddress, uint8_t Length, uint8_t* Data);

/*Force charging off*/
void ParamId308(uint8_t* ReturnCode, uint16_t ForceOff);

/*Change power mode*/
void ParamId310(uint8_t* ReturnCode, uint16_t PowerMode);

/*Get CS Status*/
#if (PLATFORM_VERSION == 1)
void ParamId346(uint8_t* ReturnCode, uint16_t* CsStatus);
#else
void ParamId346(uint8_t* ReturnCode, uint16_t* CsStatus, uint32_t TargetSerialNumber);
#endif

/*Get status CS-Mower interface*/
#if (PLATFORM_VERSION == 1)
void ParamId348(uint8_t* ReturnCode, uint16_t* StatusMowerCsInterface);
#else
void ParamId348(uint8_t* ReturnCode, uint16_t* StatusMowerCsInterface, uint32_t TargetSerialNumber);
#endif

/*Get CS Input voltage*/
#if (PLATFORM_VERSION == 1)
void ParamId354(uint8_t* ReturnCode, uint16_t* CsInputVoltage);
#else
void ParamId354(uint8_t* ReturnCode, uint16_t* CsInputVoltage, uint32_t TargetSerialNumber);
#endif

/*Get mower charging current*/
#if (PLATFORM_VERSION == 1)
void ParamId356(uint8_t* ReturnCode, uint16_t* MowerChargningCurrent);
#else
void ParamId356(uint8_t* ReturnCode, uint16_t* MowerChargningCurrent, uint32_t TargetSerialNumber);
#endif

/*Get CS temperature*/
#if (PLATFORM_VERSION == 1)
void ParamId358(uint8_t* ReturnCode, int16_t* CsTemp);
#else
void ParamId358(uint8_t* ReturnCode, int16_t* CsTemp, uint32_t TargetSerialNumber);
#endif

/*Get CS loop currents*/
#if (PLATFORM_VERSION == 1)
void ParamId360(uint8_t* ReturnCode, uint16_t* BoundaryCurrent, uint16_t* FarFieldCurrent, uint16_t* GuideWireOneCurrent, uint16_t* GuideWireTwoCurrent);
#else
void ParamId360(uint8_t* ReturnCode, uint16_t* BoundaryCurrent, uint16_t* FarFieldCurrent, uint16_t* GuideWireOneCurrent, uint16_t* GuideWireTwoCurrent, uint32_t TargetSerialNumber);
#endif

/*Enter Test mode*/
void ParamId374(uint8_t* ReturnCode, uint8_t TestMode);

/*Get Loop sensor real time data - Front Center sensor*/
#if(PLATFORM_VERSION == 1)
void ParamId394(uint8_t* ReturnCode, uint8_t* SignQual, int16_t* SignLvlBSign, int16_t* SignLvlFSign, int16_t* SignLvlG1Sign, int16_t* SignLvlG2Sign);
#else
void ParamId394(uint8_t* ReturnCode, uint8_t* SignQual, int16_t* SignLvlBSign, int16_t* SignLvlFSign, int16_t* SignLvlG1Sign, int16_t* SignLvlG2Sign, int16_t* SignNearField);
#endif

/*Get forced SW download status*/
void ParamId398(uint8_t* ReturnCode, uint16_t SwToCheck, uint16_t* DeviceGrNo, uint8_t* SubDeviceGrNo, uint8_t* VariantNo, uint8_t* SwVerMajorPart, uint8_t* SwVerMinorPart, uint32_t* SwVerBuildNo, uint8_t* DWSwVerMajorPart, uint8_t* DWSwVerMinorPart, uint32_t* DWSwVerBuildNo, uint32_t* RelativeAdress);

/*Get MCU2 serial number*/
void ParamId404(uint8_t* ReturnCode, uint8_t* Serialbuf);

/*Disable BLE communication*/
void ParamId406(uint8_t* ReturnCode, uint16_t BleDisableTime);

/*Disable cellular communication*/
void ParamId416(uint8_t* ReturnCode, uint16_t gprsLteDisableTime);

/*Disable Wi-Fi communication*/
void ParamId418(uint8_t* ReturnCode, uint16_t wifiDisableTime);

/*Set Selected temporary cellular interface*/
void ParamId424(uint8_t* ReturnCode, uint16_t CellularInterface);

/*Get Selected cellular interface*/
void ParamId428(uint8_t* ReturnCode, uint32_t* CellularInterface);

/*Set energy saving mode*/
void ParamId436(uint8_t* ReturnCode, uint8_t EnergySavingMode);

/*Get energy saving mode*/
void ParamId438(uint8_t* ReturnCode, uint8_t* EnergySavingMode);

/*Get Radar status*/
void ParamId440(uint8_t* ReturnCode, uint8_t* RadarStatus);

/*Get closest radar object*/
void ParamId442(uint8_t* ReturnCode, uint8_t* RadarStatus, uint16_t* RadarSignalStrength, int16_t* ObjectLateralDistance, int16_t* ObjectLongitudinalDistance, int16_t* ObjectHeight);

/*Reset 'Radar statistics'*/
void ParamId446(uint8_t* ReturnCode);

/*Set Enable Radar*/
void ParamId448(uint8_t* ReturnCode, uint8_t Value);

/*Get Enable Radar*/
void ParamId450(uint8_t* ReturnCode, uint8_t* enableRadar);

/*Get Radar Application SW*/
void ParamId460(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Radar Boot SW*/
void ParamId462(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Set cutting height*/
void ParamId468(uint8_t* ReturnCode, uint8_t CuttingHeight_mm);

/*Get cutting height*/
void ParamId470(uint8_t* ReturnCode, uint8_t* CuttingHeight_mm);

/*Calibrate and set cutting height*/
void ParamId472(uint8_t* ReturnCode, uint8_t CuttingHeight_mm);

/*Get Application board information*/
void ParamId526(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint32_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/*Get Radar board information*/
void ParamId528(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint32_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/*Get RA Main board (PCBA) information*/
void ParamId530(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint32_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/*Set CS Device Type*/
void ParamId532(uint8_t* ReturnCode, uint32_t CsSerialNo, uint16_t MowerDevGrNo, uint8_t MowerDevSubNo, uint8_t MowerDevVarNo);

/*Set RA Device Type*/
void ParamId534(uint8_t* ReturnCode, uint32_t RaSerialNo, uint16_t MowerDevGrNo, uint8_t MowerDevSubNo, uint8_t MowerDevVarNo);

/*Set CS production information*/
void ParamId536(uint8_t* ReturnCode, uint32_t SerialNo, uint32_t PartNo, uint16_t ModelYear, uint32_t ProdTime, uint8_t* MowerNameBuff);

/*Set RA production information*/
void ParamId538(uint8_t* ReturnCode, uint32_t SerialNo, uint32_t PartNo, uint16_t ModelYear, uint32_t ProdTime, uint8_t* MowerNameBuff);

/*Get CS production information*/
void ParamId540(uint8_t* ReturnCode, uint32_t SerialNo, uint32_t* PartNo, uint16_t* ModelYear, uint32_t* ProductionTime, uint8_t* Name, uint16_t* DeviceNo, uint8_t* SubDeviceNo, uint8_t* VariantNo, uint32_t* oSerialNo);

/*Get RA Production information*/
void ParamId542(uint8_t* ReturnCode, uint32_t SerialNo, uint32_t* PartNo, uint16_t* ModelYear, uint32_t* ProductionTime, uint8_t* Name, uint16_t* DeviceNo, uint8_t* SubDeviceNo, uint8_t* VariantNo, uint32_t* oSerialNo);

/*Get RA Application SW*/
void ParamId544(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo, uint32_t* SerialNo);

/*Get RA Boot SW*/
void ParamId546(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo, uint32_t* SerialNo);

/*Set RA LED*/
void ParamId548(uint8_t* ReturnCode, uint16_t RALed);

/*Get RA Status*/
void ParamId552(uint8_t* ReturnCode, uint32_t SerialNo, uint16_t* Status, uint8_t* RtkSignalQuality, uint8_t* NrOfSatellites, uint8_t* achv_positional_accuracy, uint32_t* ra_serial_number);

/*Get RA temperature*/
void ParamId554(uint8_t* ReturnCode, uint32_t SerialNo, int16_t* temperature);

/*Get RA wireless communication status*/
void ParamId556(uint8_t* ReturnCode, uint32_t SerialNo, uint8_t* BLEHwStat, uint8_t* BLEConnStat, uint8_t* LoraHwStat, uint8_t* LoraConStat, uint8_t* RtkHwStat,
	uint8_t* RtkSignalStat, uint8_t* NumOfRtkSats, uint8_t* GprsLteStat, uint8_t* GprsLteSignQual, uint8_t* SimStatus, uint8_t* GprsLteConnStat);

/*Get RA accelerometer data*/
void ParamId558(uint8_t* ReturnCode, uint32_t SerialNo, int16_t* PitchAngle, int16_t* RollAngle, int16_t* MowerVertGForce);

/*Add RA to RA list*/
void ParamId562(uint8_t* ReturnCode, uint32_t RaSN);

/*Speaker sound test*/
void ParamId568(uint8_t* ReturnCode, uint8_t On);

/*Get Mower Main board (PCBA) information*/
void ParamId570(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint32_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/*Get CS-board (PCBA) information*/
void ParamId572(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint32_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/*Set CS LED test*/
void ParamId574(uint8_t* ReturnCode, uint16_t LedState, uint32_t TargetSerial);

/*Get Right wheel motor encoder*/
void ParamId578(uint8_t* ReturnCode, uint8_t* EncoderValue);

/*Get Left wheel motor encoder*/
void ParamId580(uint8_t* ReturnCode, uint8_t* EncoderValue);

/*Get Height motor encoder*/
void ParamId582(uint8_t* ReturnCode, uint8_t* EncoderValue);

/*Disable function power*/
void ParamId584(uint8_t* ReturnCode, uint8_t Bits);

/*Set PCBA production information*/
void ParamId586(uint8_t* ReturnCode, uint16_t PcbDeGrNo, uint8_t PcbSubDeNo, uint8_t PcbVarNo,
	uint32_t PcbPN, uint16_t PcbRev, uint32_t PcbSerNo, uint32_t PcbProdTime, uint8_t PcbExtFlash, uint8_t PcbExtEeprom, uint8_t PcbAccelerometer);

/*Get Mower Application SW*/
void ParamId588(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Get Mower SW Package*/
void ParamId590(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/*Set Front light mode*/
void ParamId606(uint8_t* ReturnCode, uint8_t FrontLightMode, uint8_t Power);

/*Get Front light mode*/
void ParamId608(uint8_t* ReturnCode, uint8_t* FrontLightMode);

/*Set Rear light mode*/
void ParamId610(uint8_t* ReturnCode, uint8_t RearLightMode);

/*Get Rear light mode*/
void ParamId612(uint8_t* ReturnCode, uint8_t* RearLightMode);

/*Set Speaker sound*/
void ParamId614(uint8_t* ReturnCode, uint8_t SpeakerSound);

/*Get Speaker sound*/
void ParamId616(uint8_t* ReturnCode, uint8_t* SpeakerSound);

/*Set Application board digital output*/
void ParamId620(uint8_t* ReturnCode, uint8_t ApplicationBoardDigitalOut);

/*Set Main board PCBA production information*/
void ParamId626(uint8_t* ReturnCode, uint16_t PcbDeGrNo, uint8_t PcbSubDeNo, uint8_t PcbVarNo, uint32_t PcbPN, uint16_t PcbRev, uint32_t PcbSerNo, uint32_t PcbProdTime, uint8_t PcbExtFlash, uint8_t PcbExtEeprom, uint8_t PcbAccelerometer);

/*Set Radar board PCBA production information*/
void ParamId628(uint8_t* ReturnCode, uint16_t PcbDeGrNo, uint8_t PcbSubDeNo, uint8_t PcbVarNo, uint32_t PcbPN, uint16_t PcbRev, uint32_t PcbSerNo, uint32_t PcbProdTime, uint8_t PcbExtFlash, uint8_t PcbExtEeprom, uint8_t PcbAccelerometer);

/*Calibrate radar module*/
void ParamId648(uint8_t* ReturnCode, uint8_t* CalibrateRadarModule);

/*Get CS wireless communication status*/
void ParamId652(uint8_t* ReturnCode, uint8_t* BleHwStatus, uint8_t* BleConnectionStatus);

/*Get Mower Boot SW*/
void ParamId654(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/* Safety MCU RTK connection status */
void ParamId656(uint8_t* ReturnCode, uint8_t* Status);

/*Motor test*/
void ParamId658(uint8_t* ReturnCode, uint8_t MotorTest);

/* Get Camera status */
void ParamId660(uint8_t* ReturnCode, uint8_t* CameraStatus);

/* Camera parameter calibration */
void ParamId662(uint8_t* ReturnCode, uint8_t CalibMode, uint8_t* CameraCommStatus, uint8_t* CameraFocusVerification, uint8_t* CalibrationLogging);

/* Camera Inverse Perspective Mapping calibration */
void ParamId664(uint8_t* ReturnCode, uint8_t* CameraCommStatus, uint8_t* CalibrationResult);

/* Set Enable Camera */
void ParamId666(uint8_t* ReturnCode, uint8_t Value);

/* Get Enable Camera */
void ParamId668(uint8_t* ReturnCode, uint8_t* enableCamera);

/* Set Lora Config */
void ParamId692(uint8_t* ReturnCode, uint8_t SpreadSpectrumFactor, uint8_t CodeRate, uint8_t Bandwidth, uint32_t LocalID,
			 uint8_t TXPower, uint8_t TestMode, uint8_t RTCMProtocol);

/* Get Lora Config */
void ParamId694(uint8_t* ReturnCode, uint8_t* SpreadSpectrumFactor, uint8_t* CodeRate, uint16_t* RaStatus,
				   uint32_t* LocalID, uint8_t* TXPower, uint8_t* TestMode, uint8_t* RTCMProtocol);

/* Get RTK Status */
void ParamId708(uint8_t* ReturnCode, uint16_t* NumberOfSatellites, uint32_t* Pdop, uint32_t* CN0min, uint32_t* CN0avg, uint32_t* CN0max, uint16_t* NoisePerMs);

/* Get specific file from application board. */
void ParamId714(uint8_t* ReturnCode, uint8_t FileId, uint32_t MaxFileSizeRequested, uint32_t RelativeAddress, uint16_t MaxLength, uint16_t* DataLength, uint8_t* LogData);

/* Get Camera SW */
void ParamId722(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);

/* Get Camera-board (PCBA) information */
void ParamId742(uint8_t* ReturnCode, uint16_t* PcbDeGrNo, uint8_t* PcbSubDeNo, uint8_t* PcbVarNo, uint32_t* PcbPN, uint16_t* PcbRev, uint8_t* PcbSerNo, uint32_t* PcbProdTime, uint8_t* PcbExtFlash, uint8_t* PcbExtEeprom, uint8_t* PcbAccelerometer);

/* set communication board info */
void ParamId774(uint8_t* ReturnCode, uint16_t DevGrNo, uint8_t DevSubNo, uint8_t DevVarNo);

/* key test */
void ParamId776(uint8_t* ReturnCode, uint8_t Cmd, uint8_t* UpKey, uint8_t* DownKey, uint8_t* BackKey, uint8_t* ConfirmKey);

/* percetion test */
void ParamId778(uint8_t* ReturnCode, uint8_t Cmd, uint8_t* hasValue, uint32_t* TestResult);

/* pps signal test */
void ParamId780(uint8_t* ReturnCode, uint32_t* PpsSignal);

/* store upper shell test result */
void ParamId784(uint8_t* ReturnCode, uint8_t* Data, uint16_t Length);

/* store chassis test result */
void ParamId786(uint8_t* ReturnCode, uint8_t* Data, uint16_t Length);

/* void set mqtt test mode */
void ParamId788(uint8_t* ReturnCode, uint8_t MqttTestMode);

/* get upper shell test result */
void ParamId790(uint8_t* ReturnCode, uint8_t* Data, uint16_t * Length);

/* load chassis test result */
void ParamId792(uint8_t* ReturnCode, uint8_t* Data, uint16_t * Length);

/* hmi version */
void ParamId794(uint8_t *ReturnCode, uint16_t *DevGrNo, uint8_t *SubDevGrNo, uint8_t *VarNo, uint8_t *MajParSwVer, uint8_t *MinParSwVer, uint32_t *BuildNo);

/* get mqtt status */
void ParamId796(uint8_t* ReturnCode, uint8_t* MqttStatus);

/* get commboard sw */
void ParamId798(uint8_t* ReturnCode, uint8_t* version);

/* get obstacle avoidance level */
void ParamId856(uint8_t* ReturnCode, uint8_t* level);

/* get running test result */
void ParamId886(uint8_t* ReturnCode, uint8_t* Status);

/* calibrate wheels*/
void ParamId970(uint8_t* return_code, uint8_t type, uint8_t* state, uint8_t* result, uint16_t* left_wheel_angle, uint16_t* right_wheel_angle);

/* get 4g board info */
void ParamId980(uint8_t* return_code, uint8_t* imei, uint8_t* imsi, uint8_t *iccid, uint8_t* operator_name, uint32_t* part_number);

/* get accelerator data */
void ParamId990(uint8_t* return_code, int16_t* pitch, int16_t* roll, int16_t* yaw, int16_t* accel_x,
				   int16_t* accel_y, int16_t* accel_z, int16_t* gyro_x, int16_t* gyro_y, int16_t* gyro_z,
				   uint32_t* utc_sec, uint32_t* utc_usec);

/* get imu sw */
void ParamId1002(uint8_t* ReturnCode, uint16_t* DevGrNo, uint8_t* SubDevGrNo, uint8_t* VarNo, uint8_t* MajParSwVer, uint8_t* MinParSwVer, uint32_t* BuildNo);
/* set rain sensor power */
void ParamId1004(uint8_t* return_code, uint8_t power);
/* get rain sensor info */
void ParamId1006(uint8_t* return_code, uint8_t* power, uint8_t* rain_status, uint16_t* ad_value);
/* get left motor app sw */
void ParamId1008(uint8_t *ReturnCode, uint16_t *DevGrNo, uint8_t *SubDevGrNo, uint8_t *VarNo, uint8_t *MajParSwVer, uint8_t *MinParSwVer, uint32_t *BuildNo);
/* get right motor app sw */
void ParamId1010(uint8_t *ReturnCode, uint16_t *DevGrNo, uint8_t *SubDevGrNo, uint8_t *VarNo, uint8_t *MajParSwVer, uint8_t *MinParSwVer, uint32_t *BuildNo);
/* get left motor boot sw */
void ParamId1012(uint8_t *ReturnCode, uint16_t *DevGrNo, uint8_t *SubDevGrNo, uint8_t *VarNo, uint8_t *MajParSwVer, uint8_t *MinParSwVer, uint32_t *BuildNo);
/* get right motor boot sw */
void ParamId1014(uint8_t *ReturnCode, uint16_t *DevGrNo, uint8_t *SubDevGrNo, uint8_t *VarNo, uint8_t *MajParSwVer, uint8_t *MinParSwVer, uint32_t *BuildNo);
/* set hmi logo */
void ParamId1016(uint8_t* return_code, uint8_t logo);
/* set wheel motor action */
void ParamId1020(uint8_t* return_code, int16_t left_front_speed, int16_t right_front_speed, int16_t left_rear_speed,
                 int16_t right_rear_speed,
                 int8_t left_rear_angle, int8_t right_rear_angle);
/* get wheel motor info */
void ParamId1022(uint8_t* return_code, int16_t* left_front_speed, int16_t* right_front_speed,
				 int16_t* left_rear_speed, int16_t* right_rear_speed, int8_t* left_rear_angle,
				 int8_t* right_rear_angle, int16_t* left_front_drive_current,
				 int16_t* right_front_drive_current, int16_t* left_rear_drive_current,
				 int16_t* right_rear_drive_current, int16_t* left_rear_steering_current,
				 int16_t* right_rear_steering_current);

/* get chassis state */
void ParamId1024(uint8_t* ret_code, uint8_t* locked, uint32_t* wmc_error_event, uint32_t* cutting_error_event,
					uint8_t* left_steer_error_code, uint8_t* left_steer_error_detail_code,
					uint8_t* left_steer_error_state1, uint8_t* left_steer_error_state2, uint8_t* right_steer_error_code,
					uint8_t* right_steer_error_detail_code, uint8_t* right_steer_error_state1,
					uint8_t* right_steer_error_state2);

/* reset motor stall */
void ParamId1036(uint8_t* return_code, uint8_t mode);
