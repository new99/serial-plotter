double x = 0;
double k = 100;
void setup() {
  Serial.begin(9600);
}

void loop() {
  double f1 = sin(x) + sin(3*x)/3;
  double f2 = sin(2*x)/2 +sin(4*x)/4;
  double f3 = 2*f1+f2;
  double ft = int(x*100)%int(2*M_PI*100);
  if (int(ft) == 0)
    x = 0.0;
  x += 0.01;
  Serial.println(f1, 5);
  Serial.println(f2, 5);
  Serial.println(f3, 5);
  Serial.println(ft, 5);
  Serial.println();
}
