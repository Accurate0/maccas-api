Java.perform(() => {
  const utilsClass = Java.use("com.mcdonalds.androidsdk.core.util.McDUtils");
  console.log(utilsClass.getAkamaiSensorData());
});
