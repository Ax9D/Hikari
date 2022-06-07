#include <stdio.h>
#include <math.h>
#include <float.h>

void copy_v3_v3(float dest[3], const float src[3]) {
  dest[0] = src[0];
  dest[1] = src[1];
  dest[2] = src[2];
}
void compatible_eul(float eul[3], const float oldrot[3])
{
  /* we could use M_PI as pi_thresh: which is correct but 5.1 gives better results.
   * Checked with baking actions to fcurves - campbell */
  const float pi_thresh = (5.1f);
  const float pi_x2 = (2.0f * (float)M_PI);

  float deul[3];
  unsigned int i;

  /* correct differences of about 360 degrees first */
  for (i = 0; i < 3; i++) {
    deul[i] = eul[i] - oldrot[i];
    if (deul[i] > pi_thresh) {
      eul[i] -= floorf((deul[i] / pi_x2) + 0.5f) * pi_x2;
      deul[i] = eul[i] - oldrot[i];
    }
    else if (deul[i] < -pi_thresh) {
      eul[i] += floorf((-deul[i] / pi_x2) + 0.5f) * pi_x2;
      deul[i] = eul[i] - oldrot[i];
    }
  }

  /* is 1 of the axis rotations larger than 180 degrees and the other small? NO ELSE IF!! */
  if (fabsf(deul[0]) > 3.2f && fabsf(deul[1]) < 1.6f && fabsf(deul[2]) < 1.6f) {
    if (deul[0] > 0.0f) {
      eul[0] -= pi_x2;
    }
    else {
      eul[0] += pi_x2;
    }
  }
  if (fabsf(deul[1]) > 3.2f && fabsf(deul[2]) < 1.6f && fabsf(deul[0]) < 1.6f) {
    if (deul[1] > 0.0f) {
      eul[1] -= pi_x2;
    }
    else {
      eul[1] += pi_x2;
    }
  }
  if (fabsf(deul[2]) > 3.2f && fabsf(deul[0]) < 1.6f && fabsf(deul[1]) < 1.6f) {
    if (deul[2] > 0.0f) {
      eul[2] -= pi_x2;
    }
    else {
      eul[2] += pi_x2;
    }
  }
}
void mat3_normalized_to_eul2(const float mat[3][3], float eul1[3], float eul2[3])
{
  const float cy = hypotf(mat[0][0], mat[0][1]);

  if (cy > 16.0f * FLT_EPSILON) {

    eul1[0] = atan2f(mat[1][2], mat[2][2]);
    eul1[1] = atan2f(-mat[0][2], cy);
    eul1[2] = atan2f(mat[0][1], mat[0][0]);

    eul2[0] = atan2f(-mat[1][2], -mat[2][2]);
    eul2[1] = atan2f(-mat[0][2], -cy);
    eul2[2] = atan2f(-mat[0][1], -mat[0][0]);
  }
  else {
    eul1[0] = atan2f(-mat[2][1], mat[1][1]);
    eul1[1] = atan2f(-mat[0][2], cy);
    eul1[2] = 0.0f;

    copy_v3_v3(eul2, eul1);
  }
}
/* skip error check, currently only needed by mat3_to_quat_is_ok */
void quat_to_mat3_no_error(float m[3][3], const float q[4])
{
  double q0, q1, q2, q3, qda, qdb, qdc, qaa, qab, qac, qbb, qbc, qcc;

  q0 = M_SQRT2 * (double)q[0];
  q1 = M_SQRT2 * (double)q[1];
  q2 = M_SQRT2 * (double)q[2];
  q3 = M_SQRT2 * (double)q[3];

  qda = q0 * q1;
  qdb = q0 * q2;
  qdc = q0 * q3;
  qaa = q1 * q1;
  qab = q1 * q2;
  qac = q1 * q3;
  qbb = q2 * q2;
  qbc = q2 * q3;
  qcc = q3 * q3;

  m[0][0] = (float)(1.0 - qbb - qcc);
  m[0][1] = (float)(qdc + qab);
  m[0][2] = (float)(-qdb + qac);

  m[1][0] = (float)(-qdc + qab);
  m[1][1] = (float)(1.0 - qaa - qcc);
  m[1][2] = (float)(qda + qbc);

  m[2][0] = (float)(qdb + qac);
  m[2][1] = (float)(-qda + qbc);
  m[2][2] = (float)(1.0 - qaa - qbb);
}
void quat_to_mat3(float m[3][3], const float q[4])
{
#ifdef DEBUG
  float f;
  if (!((f = dot_qtqt(q, q)) == 0.0f || (fabsf(f - 1.0f) < (float)QUAT_EPSILON))) {
    fprintf(stderr,
            "Warning! quat_to_mat3() called with non-normalized: size %.8f *** report a bug ***\n",
            f);
  }
#endif

  quat_to_mat3_no_error(m, q);
}
/* uses 2 methods to retrieve eulers, and picks the closest */

/* XYZ order */
void mat3_normalized_to_compatible_eul(float eul[3], const float oldrot[3], float mat[3][3])
{
  float eul1[3], eul2[3];
  float d1, d2;

  mat3_normalized_to_eul2(mat, eul1, eul2);

  compatible_eul(eul1, oldrot);
  compatible_eul(eul2, oldrot);

  d1 = fabsf(eul1[0] - oldrot[0]) + fabsf(eul1[1] - oldrot[1]) + fabsf(eul1[2] - oldrot[2]);
  d2 = fabsf(eul2[0] - oldrot[0]) + fabsf(eul2[1] - oldrot[1]) + fabsf(eul2[2] - oldrot[2]);

  /* return best, which is just the one with lowest difference */
  if (d1 > d2) {
    copy_v3_v3(eul, eul2);
  }
  else {
    copy_v3_v3(eul, eul1);
  }
}


void eul_to_mat3(float mat[3][3], const float eul[3])
{
  double ci, cj, ch, si, sj, sh, cc, cs, sc, ss;

  ci = cos(eul[0]);
  cj = cos(eul[1]);
  ch = cos(eul[2]);
  si = sin(eul[0]);
  sj = sin(eul[1]);
  sh = sin(eul[2]);
  cc = ci * ch;
  cs = ci * sh;
  sc = si * ch;
  ss = si * sh;

  mat[0][0] = (float)(cj * ch);
  mat[1][0] = (float)(sj * sc - cs);
  mat[2][0] = (float)(sj * cc + ss);
  mat[0][1] = (float)(cj * sh);
  mat[1][1] = (float)(sj * ss + cc);
  mat[2][1] = (float)(sj * cs - sc);
  mat[0][2] = (float)-sj;
  mat[1][2] = (float)(cj * si);
  mat[2][2] = (float)(cj * ci);
}
void quat_to_compatible_eul(float eul[3], const float oldrot[3], const float quat[4])
{
  float unit_mat[3][3];
  quat_to_mat3(unit_mat, quat);
  mat3_normalized_to_compatible_eul(eul, oldrot, unit_mat);
}

void print_mat(float mat[3][3]) {
  for(int i = 0; i < 3; i++) {
      for(int j = 0; j < 3; j++) {
        printf("%f ", mat[i][j]);
      }
      printf("\n");
  }
}
/* XYZ order */
void eul_to_quat(float quat[4], const float eul[3])
{
  float ti, tj, th, ci, cj, ch, si, sj, sh, cc, cs, sc, ss;

  ti = eul[0] * 0.5f;
  tj = eul[1] * 0.5f;
  th = eul[2] * 0.5f;
  ci = cosf(ti);
  cj = cosf(tj);
  ch = cosf(th);
  si = sinf(ti);
  sj = sinf(tj);
  sh = sinf(th);
  cc = ci * ch;
  cs = ci * sh;
  sc = si * ch;
  ss = si * sh;

  quat[0] = cj * cc + sj * ss;
  quat[1] = cj * sc - sj * cs;
  quat[2] = cj * ss + sj * cc;
  quat[3] = cj * cs - sj * sc;
}

const float RAD2DEG = 180.0f / M_PI;
const float DEG2RAD = 1.0f / RAD2DEG;

int main() {
  const float STEP = 0.1f;
  for(float x = 0.0f; x <= 360.0f; x+=STEP) {
      for(float y = 0.0f; y <= 360.0f; y+=STEP) {
          for(float z = 0.0f; z <= 360.0f; z+=STEP) {
              float quat[3] = {0};
              float eul[3] = {x * DEG2RAD, y * DEG2RAD, z * DEG2RAD};
              eul_to_quat(quat, eul);
              float new_eul[3] = {0};
              quat_to_compatible_eul(new_eul, eul, quat);

              printf("(%f, %f, %f) => (%f, %f, %f)\n", eul[0] * RAD2DEG, eul[1] * RAD2DEG, eul[2] * RAD2DEG, new_eul[0] * RAD2DEG, new_eul[1] * RAD2DEG, new_eul[2] * RAD2DEG);
          }
      }
  }
}
