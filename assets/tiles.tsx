<?xml version="1.0" encoding="UTF-8"?>
<tileset name="Tiles" tilewidth="64" tileheight="64" tilecount="3" columns="0">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0">
  <properties>
   <property name="ladder" type="bool" value="false"/>
   <property name="solid" type="bool" value="true"/>
  </properties>
  <image width="64" height="64" source="tiles/brick.png"/>
 </tile>
 <tile id="1">
  <properties>
   <property name="ladder" type="bool" value="true"/>
   <property name="solid" type="bool" value="false"/>
  </properties>
  <image width="64" height="64" source="tiles/ladder.png"/>
 </tile>
 <tile id="2">
  <properties>
   <property name="ladder" type="bool" value="false"/>
   <property name="solid" type="bool" value="false"/>
  </properties>
  <image width="64" height="64" source="tiles/water.png"/>
 </tile>
</tileset>
